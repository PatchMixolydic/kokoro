#![feature(decl_macro)]
#![feature(iter_intersperse)]
#![feature(once_cell)]
#![feature(pattern)]

mod models;
mod parser;

use std::{env, lazy::SyncLazy};

use async_std::task;
use futures_lite::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

/// The name that the bot will give to generated webhooks.
static BOT_WEBHOOK_NAME: SyncLazy<String> =
    SyncLazy::new(|| env::var("BOT_WEBHOOK_NAME").unwrap_or_else(|_| "Kokoro".to_owned()));

static BOT_USER_ID: SyncLazy<u64> = SyncLazy::new(|| {
    env::var("BOT_USER_ID")
        .expect("The BOT_USER_ID environment variable must be set (preferably in .env).")
        .parse()
        .expect("The BOT_USER_ID environment variable must contain a 64-bit unsigned integer.")
});

static TOKEN: SyncLazy<String> = SyncLazy::new(|| {
    env::var("BOT_TOKEN")
        .expect("The BOT_TOKEN environment variable must be set (preferably in .env).")
});

async fn handle_event(shard_id: u64, event: Event, http: HttpClient) -> anyhow::Result<()> {
    match event {
        Event::MessageCreate(msg) => match parser::parse_command(&msg).await {
            Some(reply) => {
                http.create_message(msg.channel_id).content(reply)?.await?;
            },

            None => {
                parser::parse_prefix(&http, &msg).await;
            },
        },

        Event::ShardConnected(_) => {
            println!("Connected on shard {}.", shard_id);
        },

        _ => {},
    }

    Ok(())
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    models::run_migrations().await;

    let cluster = Cluster::builder(&*TOKEN, Intents::GUILD_MESSAGES)
        .shard_scheme(ShardScheme::Auto)
        .build()
        .await?;

    let cluster_spawn = cluster.clone();

    task::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = HttpClient::new(&*TOKEN);
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let mut events = cluster.events();

    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        task::spawn(handle_event(shard_id, event, http.clone()));
    }

    Ok(())
}
