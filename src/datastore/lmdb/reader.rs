use crate::{
    datastore::Result,
    sessions::{SessionData, SessionId},
};

use super::{do_async, schema::Schema};

pub struct Reader {
    schema: Schema,
    mode: ReadMode,
}

impl Reader {
    pub fn new(schema: Schema, mode: ReadMode) -> Self {
        Self { schema, mode }
    }

    pub async fn read_session(&self, id: SessionId) -> Result<Option<SessionData>> {
        match self.mode {
            ReadMode::Sync => self.schema.read_session(id),
            ReadMode::Async => {
                let schema = self.schema.clone();
                do_async(move || schema.read_session(id)).await
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ReadMode {
    #[default]
    Sync,
    Async,
}
