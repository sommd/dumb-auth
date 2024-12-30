use std::{collections::HashMap, time::Instant};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::sync::RwLock;

use crate::config::SessionExpiry;

pub struct Sessions {
    expiry: SessionExpiry,
    sessions: RwLock<HashMap<String, Session>>,
}

impl Sessions {
    pub fn new(expiry: SessionExpiry) -> Self {
        Self {
            expiry,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_valid_session(&self, token: &str) -> Option<Session> {
        let session = {
            let r = self.sessions.read().await;
            r.get(token)?.clone()
        };

        if let SessionExpiry::Duration(expiry) = self.expiry {
            if session.created.elapsed() >= expiry {
                self.sessions.write().await.remove(token);
                return None;
            }
        }

        Some(session)
    }

    pub async fn create_session(&self) -> (String, Session) {
        let session = Session {
            created: Instant::now(),
        };

        let mut token = generate_token();
        {
            let mut w = self.sessions.write().await;
            // Should never happen, but just make sure we generate a unique token
            while w.contains_key(&token) {
                token = generate_token();
            }
            w.insert(token.clone(), session.clone());
        }

        (token, session)
    }
}

#[derive(Clone)]
pub struct Session {
    created: Instant,
}

impl Session {
    pub fn created(&self) -> Instant {
        self.created
    }
}

fn generate_token() -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}
