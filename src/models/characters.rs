use super::{ModelError, DB_POOL};

/// Represents a character in the database
#[derive(Debug)]
pub struct Character {
    pub char_id: i64,
    pub user_id: i64,
    pub char_name: String,
    pub char_avatar: String,
    pub char_prefix: String,
}

impl Character {
    /// Returns information about the character with the given id in the
    /// database or an error if something went wrong.
    pub async fn with_id(char_id: u32) -> Result<Self, ModelError> {
        let res = sqlx::query_as!(
            Character,
            "select * from characters where char_id = $1 limit 1",
            char_id
        )
        .fetch_one(&*DB_POOL)
        .await?;

        Ok(res)
    }

    /// Returns the character with the given name for this user
    /// or an error if something went wrong.
    pub async fn with_name_for_user(name: &str, user_id: u64) -> Result<Self, ModelError> {
        // Cast done here since this is an implementation detail.
        // sqlite does not accept `u64`s.
        let user_id = user_id as i64;

        let res = sqlx::query_as!(
            Character,
            "select * from characters where (char_name = $1) and (user_id = $2) limit 1",
            name,
            user_id
        )
        .fetch_one(&*DB_POOL)
        .await?;

        Ok(res)
    }

    /// Returns information about all characters for the given user.
    pub async fn all_with_user_id(user_id: u64) -> Result<Vec<Self>, ModelError> {
        // Cast done here since this is an implementation detail.
        // sqlite does not accept `u64`s.
        let user_id = user_id as i64;

        let res = sqlx::query_as!(
            Character,
            "select * from characters where user_id = $1",
            user_id
        )
        .fetch_all(&*DB_POOL)
        .await?;

        Ok(res)
    }

    /// Create a new character.
    pub async fn insert(
        user_id: u64,
        name: &str,
        avatar: &str,
        prefix: &str,
    ) -> Result<(), ModelError> {
        // Cast done here since this is an implementation detail.
        // sqlite does not accept `u64`s.
        let user_id = user_id as i64;

        sqlx::query!(
            "insert into characters
            (user_id, char_name, char_avatar, char_prefix)
            values ($1, $2, $3, $4)",
            user_id,
            name,
            avatar,
            prefix
        )
        .execute(&*DB_POOL)
        .await?;

        Ok(())
    }
}
