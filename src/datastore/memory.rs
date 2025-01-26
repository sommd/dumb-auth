use std::collections::{hash_map::Entry, HashMap};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    datastore::{Datastore, StoreSessionError},
    sessions::Session,
};

use super::DatastoreError;

#[derive(Default)]
pub struct InMemoryDatastore {
    sessions: RwLock<HashMap<String, Session>>,
}

impl InMemoryDatastore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Datastore for InMemoryDatastore {
    async fn store_session(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Result<(), super::StoreSessionError>, DatastoreError> {
        Ok(match self.sessions.write().await.entry(token.to_string()) {
            Entry::Occupied(_) => Err(StoreSessionError::AlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(session.clone());
                Ok(())
            }
        })
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, DatastoreError> {
        Ok(self.sessions.read().await.get(token).cloned())
    }

    async fn delete_session(&self, token: &str) -> Result<bool, DatastoreError> {
        Ok(self.sessions.write().await.remove(token).is_some())
    }
}
