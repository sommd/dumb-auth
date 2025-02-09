use std::path::Path;

use thiserror::Error;

use crate::sessions::{SessionData, SessionId};

use self::lmdb::LmdbDatastore;
use self::memory::InMemoryDatastore;

mod lmdb;
mod memory;

type Result<T> = std::result::Result<T, DatastoreError>;

pub struct Datastore(DatastoreInner);

enum DatastoreInner {
    InMemory(InMemoryDatastore),
    Lmdb(LmdbDatastore),
}

impl Datastore {
    pub fn new_in_memory() -> Self {
        Self(DatastoreInner::InMemory(InMemoryDatastore::new()))
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self(DatastoreInner::Lmdb(LmdbDatastore::open(
            path.as_ref(),
        )?)))
    }

    pub(crate) async fn create_session(&self, data: SessionData) -> Result<SessionId> {
        Ok(match &self.0 {
            DatastoreInner::InMemory(inner) => inner.create_session(data).await,
            DatastoreInner::Lmdb(inner) => inner.create_session(data).await?,
        })
    }

    pub(crate) async fn read_session(&self, id: SessionId) -> Result<Option<SessionData>> {
        Ok(match &self.0 {
            DatastoreInner::InMemory(inner) => inner.read_session(id).await,
            DatastoreInner::Lmdb(inner) => inner.read_session(id).await?,
        })
    }

    pub(crate) async fn delete_session(&self, id: SessionId) -> Result<bool> {
        Ok(match &self.0 {
            DatastoreInner::InMemory(inner) => inner.delete_session(id).await,
            DatastoreInner::Lmdb(inner) => inner.delete_session(id).await?,
        })
    }
}

#[derive(Debug, Error)]
pub enum DatastoreError {
    #[error("{0}")]
    HeedError(#[from] heed::Error),
    #[error("file does not appear to be a dumb-auth datastore")]
    UnrecognizedFormat,
    #[error("unknown datastore version: {0}")]
    UnknownVersion(u64),
    #[error("datastore is corrupted")]
    Corrupt,
}
