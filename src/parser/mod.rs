mod macros;

use chrono::{DateTime, Utc};
use std::lazy::SyncLazy;
use twilight_command_parser::{Command, CommandParserConfig, Parser};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::MessageCreate;
use webhook::Webhook;

use crate::{
    models::{characters::Character, webhooks::get_or_create_webhook},
    parser, BOT_USER_ID,
};

parser! {
    with (message, arguments) {
        echo(..message) {
            message
        }

        ping() {
            let message_timestamp = DateTime::parse_from_rfc3339(&message.timestamp).unwrap();
            let time = Utc::now().signed_duration_since(message_timestamp).num_milliseconds();
            format!("Pong. Command recieved in **{}** ms.", time)
        }

        ls() {
            let characters = Character::all_with_user_id(message.author.id.0).await.unwrap();

            if characters.is_empty() {
                "You have no characters.".to_owned()
            } else {
                characters
                    .iter()
                    .map(|ch| format!("{} (prefix `{}`)", ch.char_name, ch.char_prefix))
                    .intersperse("\n".to_owned())
                    .collect()
            }
        }

        create(name, avatar, prefix) {
            Character::insert(message.author.id.0, name, avatar, prefix).await.unwrap();
            format!("Created {} successfully.", name)
        }
    }
}

/// Parse a message, checking to see if it contains a prefix that matches
/// one of the message author's characters.
pub async fn parse_prefix(http: &HttpClient, message: &MessageCreate) {
    let characters = Character::all_with_user_id(message.author.id.0)
        .await
        .unwrap();

    for character in characters {
        if let Some(message_no_prefix) = message.content.strip_prefix(&character.char_prefix) {
            let webhook = Webhook::from_url(
                &get_or_create_webhook(&http, message.channel_id.0)
                    .await
                    .unwrap(),
            );

            webhook
                .send(|res| {
                    res.username(&character.char_name)
                        .avatar_url(&character.char_avatar)
                        .content(message_no_prefix.trim())
                })
                .await
                .unwrap();

            http.delete_message(message.channel_id, message.id)
                .await
                .unwrap();

            // We're done
            return;
        }
    }
}
