use async_trait::async_trait;
use thiserror::Error;

use crate::sessions::Session;

#[cfg(feature = "lmdb")]
pub use self::lmdb::LmdbDatastore;
pub use self::memory::InMemoryDatastore;
#[cfg(any(feature = "sqlite", feature = "sqlite-unbundled"))]
pub use self::sqlite::SqliteDatastore;

#[cfg(feature = "lmdb")]
mod lmdb;
mod memory;
#[cfg(any(feature = "sqlite", feature = "sqlite-unbundled"))]
mod sqlite;

#[async_trait]
pub trait Datastore: Send + Sync {
    async fn store_session(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Result<(), StoreSessionError>, DatastoreError>;

    async fn get_session(&self, token: &str) -> Result<Option<Session>, DatastoreError>;

    async fn delete_session(&self, token: &str) -> Result<bool, DatastoreError>;
}

#[derive(Debug, Error)]
pub enum DatastoreError {
    #[cfg(any(feature = "sqlite", feature = "sqlite-unbundled"))]
    #[error("{0}")]
    SqlxError(#[from] sqlx::Error),
    #[cfg(feature = "lmdb")]
    #[error("{0}")]
    HeedError(#[from] heed::Error),
}

#[derive(Clone, Copy, Debug, Error)]
pub enum StoreSessionError {
    #[error("already exists")]
    AlreadyExists,
}
