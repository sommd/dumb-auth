use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::sessions::Session;

use super::{Datastore, DatastoreError, StoreSessionError};

pub struct SqliteDatastore {
    pool: SqlitePool,
}

impl SqliteDatastore {
    pub async fn connect(url: &str) -> sqlx::Result<Self> {
        Self::init(SqlitePool::connect(url).await?).await
    }

    pub async fn init(pool: SqlitePool) -> sqlx::Result<Self> {
        sqlx::migrate!("src/datastore/migrations")
            .run(&pool)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl Datastore for SqliteDatastore {
    async fn store_session(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Result<(), StoreSessionError>, DatastoreError> {
        let result = sqlx::query("INSERT OR IGNORE INTO sessions (token, created) VALUES (?, ?)")
            .bind(token)
            .bind(
                i64::try_from(
                    session
                        .created
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                )
                .unwrap_or(i64::MAX),
            )
            .execute(&self.pool)
            .await?;

        Ok(if result.rows_affected() == 0 {
            Err(StoreSessionError::AlreadyExists)
        } else {
            Ok(())
        })
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, DatastoreError> {
        Ok(
            sqlx::query_as("SELECT (created) FROM sessions WHERE token = ?")
                .bind(token)
                .fetch_optional(&self.pool)
                .await?
                .map(|(created,): (i64,)| Session {
                    created: SystemTime::UNIX_EPOCH
                        + Duration::from_secs(u64::try_from(created).unwrap_or(0)),
                }),
        )
    }

    async fn delete_session(&self, token: &str) -> Result<bool, DatastoreError> {
        let result = sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() != 0)
    }
}
