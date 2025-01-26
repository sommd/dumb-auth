use async_trait::async_trait;
use thiserror::Error;

use crate::sessions::Session;

pub use self::memory::InMemoryDatastore;

mod memory;

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
pub enum DatastoreError {}

#[derive(Clone, Copy, Debug, Error)]
pub enum StoreSessionError {
    #[error("already exists")]
    AlreadyExists,
}
