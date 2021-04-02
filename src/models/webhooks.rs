use twilight_http::Client as HttpClient;

use super::{ModelError, DB_POOL};
use crate::BOT_WEBHOOK_NAME;

/// Retrieves the webhook URL associated with a given channel.
/// Returns `Err(`[`ModelError::NoSuchItem`]`)` if there is no
/// webhook for this channel.
pub async fn webhook_for_channel(channel_id: u64) -> Result<String, ModelError> {
    // Cast done here since this is an implementation detail.
    let channel_id = channel_id as i64;

    let res = sqlx::query!(
        "select * from channel_to_webhook where channel_id = $1 limit 1",
        channel_id
    )
    .fetch_one(&*DB_POOL)
    .await?;

    Ok(res.webhook_url)
}

/// Retrieves the webhook URL associated with a given channel.
/// If there is no webhook for this channel, one is created
/// and saved in the database.
pub async fn get_or_create_webhook(
    http: &HttpClient,
    channel_id: u64,
) -> Result<String, ModelError> {
    let mut res = webhook_for_channel(channel_id).await;

    if let Err(ModelError::NoSuchItem) = res {
        let webhook = http
            .create_webhook(channel_id.into(), &*BOT_WEBHOOK_NAME)
            .await
            .unwrap();

        let url = format!(
            "https://discord.com/api/webhooks/{}/{}",
            webhook.id,
            webhook.token.unwrap_or_else(String::new)
        );

        // SQLite doesn't accept u64s
        let sql_channel_id = channel_id as i64;

        sqlx::query!(
            "insert into channel_to_webhook
            (channel_id, webhook_url)
            values ($1, $2)",
            sql_channel_id,
            url
        )
        .execute(&*DB_POOL)
        .await?;

        res = Ok(url);
    }

    res
}
