pub mod characters;
pub mod webhooks;

use async_std::task::block_on;
use dotenv::dotenv;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, lazy::SyncLazy};
use thiserror::Error;

pub(self) static DB_POOL: SyncLazy<SqlitePool> = SyncLazy::new(|| {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("The DATABASE_URL environment variable must be set (preferably in .env)");

    block_on(
        SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
    )
    .expect("Failed to create database connection pool")
});

#[derive(Clone, Copy, Debug, Error)]
pub enum ModelError {
    #[error("An issue with the connection to the database occurred.")]
    ConnectionError,
    #[error("An error occurred while querying the database.")]
    QueryDatabaseError,
    #[error("No such item found in the database.")]
    NoSuchItem,
    #[error("An error occurred while running database migrations.")]
    MigrationError,
}

impl From<sqlx::Error> for ModelError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Configuration(_) => Self::ConnectionError,
            sqlx::Error::Tls(_) => Self::ConnectionError,
            sqlx::Error::Protocol(_) => Self::QueryDatabaseError,
            sqlx::Error::RowNotFound => Self::NoSuchItem,
            sqlx::Error::PoolTimedOut => Self::ConnectionError,
            sqlx::Error::PoolClosed => Self::ConnectionError,
            sqlx::Error::WorkerCrashed => Self::ConnectionError,
            sqlx::Error::Migrate(_) => Self::MigrationError,
            _ => Self::QueryDatabaseError,
        }
    }
}

pub async fn run_migrations() {
    // sqlx::migrate! embeds migrations into the binary
    sqlx::migrate!().run(&*DB_POOL)
        .await
        .unwrap_or_else(|err| panic!("Failed to run migrations: {}", err));
}
