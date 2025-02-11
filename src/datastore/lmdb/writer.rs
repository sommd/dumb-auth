use std::thread;

use tokio::{
    runtime::{Handle, RuntimeFlavor},
    sync::{mpsc, oneshot},
    task,
};

use crate::{
    datastore::Result,
    sessions::{SessionData, SessionId},
};

use super::{do_async, schema::Schema};

pub struct Writer(Inner);

enum Inner {
    Sync(Schema),
    Async(Schema),
    AsyncThread(mpsc::Sender<WriteOp>),
}

type WriteRet<T> = oneshot::Sender<Result<T>>;

enum WriteOp {
    CreateSession(SessionData, WriteRet<SessionId>),
    DeleteSession(SessionId, WriteRet<bool>),
}

impl Writer {
    pub fn new(schema: Schema, mode: WriteMode) -> Self {
        match mode {
            WriteMode::Sync => Self(Inner::Sync(schema)),
            WriteMode::Async => Self(Inner::Async(schema)),
            WriteMode::AsyncThread => {
                let (tx, rx) = mpsc::channel(1);
                Self::spawn_write_thread(schema, rx);
                Self(Inner::AsyncThread(tx))
            }
        }
    }

    fn spawn_write_thread(schema: Schema, mut rx: mpsc::Receiver<WriteOp>) {
        thread::spawn(move || {
            while let Some(op) = rx.blocking_recv() {
                match op {
                    WriteOp::CreateSession(data, ret) => {
                        let _ = ret.send(schema.create_session(data));
                    }
                    WriteOp::DeleteSession(id, ret) => {
                        let _ = ret.send(schema.delete_session(id));
                    }
                }
            }
        });
    }

    pub async fn create_session(&self, data: SessionData) -> Result<SessionId> {
        match &self.0 {
            Inner::Sync(schema) => do_sync(|| schema.create_session(data)),
            Inner::Async(schema) => {
                let schema = schema.clone();
                do_async(move || schema.create_session(data)).await
            }
            Inner::AsyncThread(op_tx) => {
                do_op(op_tx, |ret| WriteOp::CreateSession(data, ret)).await
            }
        }
    }

    pub async fn delete_session(&self, id: SessionId) -> Result<bool> {
        match &self.0 {
            Inner::Sync(schema) => schema.delete_session(id),
            Inner::Async(schema) => {
                let schema = schema.clone();
                do_async(move || schema.delete_session(id)).await
            }
            Inner::AsyncThread(op_tx) => do_op(op_tx, |ret| WriteOp::DeleteSession(id, ret)).await,
        }
    }
}

fn do_sync<T>(f: impl FnOnce() -> T) -> T {
    if Handle::try_current().is_ok_and(|h| h.runtime_flavor() != RuntimeFlavor::CurrentThread) {
        task::block_in_place(f)
    } else {
        f()
    }
}

async fn do_op<T>(
    op_tx: &mpsc::Sender<WriteOp>,
    f: impl FnOnce(oneshot::Sender<T>) -> WriteOp,
) -> T {
    let (ret_tx, ret_rx) = oneshot::channel();
    op_tx.send(f(ret_tx)).await.unwrap();
    ret_rx.await.unwrap()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum WriteMode {
    Sync,
    Async,
    #[default]
    AsyncThread,
}
