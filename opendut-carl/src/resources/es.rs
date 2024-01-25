use slotmap::{DefaultKey, SlotMap};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use opendut_resources::repository::Repository;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};

type Subscriber = mpsc::Sender<Event>;

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
        resources: Vec<Resource>,
        reply_to: oneshot::Sender<CommitReply>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, PartialEq)]
pub enum CommitReply {
    Success
}

#[derive(Debug, PartialEq)]
pub enum UpdateReply {
    Success
}

#[derive(Debug, PartialEq)]
pub enum ApplyReply {
    Success
}

#[derive(Debug)]
pub enum Resource {
    ClusterConfiguration(ClusterConfiguration),
    ClusterDeployment(ClusterDeployment),
}

pub type Error = String;

#[derive(Debug)]
pub struct Snapshot {}

#[derive(Default)]
struct RM {
    storage: Repository<64>,
    subscribers: SlotMap<DefaultKey, Subscriber>
}

impl RM {

    pub fn create() -> (mpsc::Sender<Command>, JoinHandle<()>) {
        let (sender, mut receiver) = mpsc::channel::<Command>(64); // TODO: create configuration for the channel's buffer size.
        let mut rm: RM = Default::default();
        let handle = tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                rm.receive(command)
            }
        });
        (sender, handle)
    }

    fn receive(&mut self, command: Command) {
        match command {
            Command::Subscribe { subscriber } => {
                let key = self.subscribers.insert(subscriber);
                log::info!("New subscription '{key:?}'.");
            }
            Command::Snapshot { .. } => {}
            Command::Commit { resources, reply_to } => {
                for resource in resources {
                    // self.storage.commit(resource).unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use googletest::prelude::*;

    use opendut_types::cluster::{ClusterId, ClusterName};
    use opendut_types::peer::PeerId;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test() -> Result<()> {

        opendut_util::logging::initialize()?;

        let (mut resources_manager, rm_handle) = RM::create();

        let cluster_configuration = ClusterConfiguration {
            id: ClusterId::random(),
            name: ClusterName::try_from("test").unwrap(),
            leader: PeerId::random(),
            devices: Default::default(),
        };

        let mut probe = probe::mpsc::<Event>();

        resources_manager.send(Command::Subscribe {
            subscriber: probe.sender()
        }).await?;

        let mut probe = probe::oneshot();

        resources_manager.send(Command::Commit {
            resources: vec![Resource::ClusterConfiguration(cluster_configuration)],
            reply_to: probe.sender()
        }).await?;

        Ok(())
    }

    mod probe {

        pub fn oneshot<A>() -> oneshot::Probe<A> {
            oneshot::Probe::new()
        }

        pub fn mpsc<A>() -> mpsc::Probe<A> {
            mpsc::Probe::new()
        }

        mod mpsc {
            use std::time::Duration;

            use tokio::sync::mpsc;
            use tokio::time::timeout;

            pub struct Probe<A> {
                sender: mpsc::Sender<A>,
                receiver: mpsc::Receiver<A>
            }

            impl<A> Probe<A> {

                pub fn new() -> Probe<A> {
                    let (sender, receiver) = mpsc::channel(1);
                    Probe {
                        sender,
                        receiver,
                    }
                }

                pub fn sender(&self) -> mpsc::Sender<A> {
                    Clone::clone(&self.sender)
                }

                pub async fn receive_message(&mut self) -> A {
                    timeout(Duration::from_secs(3), self.receiver.recv()).await
                        .expect("Expect message")
                        .expect("Channel closed")
                }
            }
        }

        mod oneshot {
            use std::time::Duration;

            use tokio::sync::oneshot;
            use tokio::time::timeout;

            pub struct Probe<A> {
                sender: Option<oneshot::Sender<A>>,
                receiver: oneshot::Receiver<A>
            }

            impl<A> Probe<A> {

                pub fn new() -> Probe<A> {
                    let (sender, receiver) = oneshot::channel();
                    Probe {
                        sender: Some(sender),
                        receiver,
                    }
                }

                pub fn sender(&mut self) -> oneshot::Sender<A> {
                    self.sender.take()
                        .unwrap()
                }

                pub async fn receive_message(self) -> A {
                    timeout(Duration::from_secs(3), self.receiver).await
                        .expect("Expect message")
                        .expect("Channel closed")
                }
            }
        }
    }
}
