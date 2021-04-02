CREATE TABLE IF NOT EXISTS channel_to_webhook(
    channel_id BIGINT NOT NULL UNIQUE,
    webhook_url TEXT NOT NULL UNIQUE
);
