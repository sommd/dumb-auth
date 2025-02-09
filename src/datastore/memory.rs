use std::{
    collections::{hash_map::Entry, HashMap},
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::sync::RwLock;

use crate::sessions::{SessionData, SessionId};

pub struct InMemoryDatastore {
    counter: AtomicU64,
    sessions: RwLock<HashMap<SessionId, SessionData>>,
}

impl InMemoryDatastore {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
            sessions: Default::default(),
        }
    }
}

impl Default for InMemoryDatastore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryDatastore {
    pub async fn create_session(&self, data: SessionData) -> SessionId {
        let id = SessionId(self.counter.fetch_add(1, Ordering::Relaxed));

        match self.sessions.write().await.entry(id) {
            Entry::Vacant(entry) => entry.insert(data),
            Entry::Occupied(_) => panic!("ran out of session IDs, this should never happen"),
        };

        id
    }

    pub async fn read_session(&self, id: SessionId) -> Option<SessionData> {
        self.sessions.read().await.get(&id).cloned()
    }

    pub async fn delete_session(&self, id: SessionId) -> bool {
        self.sessions.write().await.remove(&id).is_some()
    }
}
