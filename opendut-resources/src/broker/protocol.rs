use tokio::sync::{mpsc, oneshot};
use crate::broker::{Event, Snapshot};

#[derive()]
pub enum Command {
    Subscribe {
        subscriber: mpsc::Sender<Event>,
        // reply_to: oneshot::Sender<SubscribeReply>,
    },
    Snapshot {
        reply_to: oneshot::Sender<Snapshot>
    },
    Commit {
        resources: Vec<String>,
        reply_to: oneshot::Sender<CommitReply>,
    },
}

#[derive(Debug, PartialEq)]
pub enum CommitReply {
    Success
}
