CREATE TABLE IF NOT EXISTS characters(
    char_id INTEGER NOT NULL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    char_name TEXT NOT NULL,
    char_avatar TEXT NOT NULL,
    char_prefix TEXT NOT NULL
);
