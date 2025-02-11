use std::{fs::File, panic, path::Path};

use heed::{EnvFlags, EnvOpenOptions};
use tokio::task;

use crate::{
    datastore::Result,
    sessions::{SessionData, SessionId},
};

pub use self::{reader::ReadMode, writer::WriteMode};
use self::{reader::Reader, schema::Schema, writer::Writer};

mod reader;
mod schema;
mod writer;

pub struct LmdbDatastore {
    reader: Reader,
    writer: Writer,
}

impl LmdbDatastore {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with(path, ReadMode::default(), WriteMode::default())
    }

    pub fn open_with(path: &Path, read_mode: ReadMode, write_mode: WriteMode) -> Result<Self> {
        let is_new = match File::options()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)
        {
            Ok(f) => f.metadata().map_err(heed::Error::Io)?.len() == 0,
            Err(e) => return Err(heed::Error::Io(e).into()),
        };

        let env = unsafe {
            EnvOpenOptions::new()
                .max_dbs(Schema::NUM_DBS)
                .map_size(4 * 1024 * 1024)
                .flags(EnvFlags::NO_SUB_DIR)
                .open(path)?
        };

        let schema = if is_new {
            Schema::init(env)
        } else {
            Schema::check(env)
        }?;

        Ok(Self {
            reader: Reader::new(schema.clone(), read_mode),
            writer: Writer::new(schema, write_mode),
        })
    }

    pub async fn create_session(&self, data: SessionData) -> Result<SessionId> {
        self.writer.create_session(data).await
    }

    pub async fn read_session(&self, id: SessionId) -> Result<Option<SessionData>> {
        self.reader.read_session(id).await
    }

    pub async fn delete_session(&self, id: SessionId) -> Result<bool> {
        self.writer.delete_session(id).await
    }
}

async fn do_async<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    task::spawn_blocking(f)
        .await
        .unwrap_or_else(|e| panic::resume_unwind(e.into_panic()))
}
