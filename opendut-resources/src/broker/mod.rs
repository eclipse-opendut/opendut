use slotmap::{DefaultKey, SlotMap};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tracing::info;

pub use protocol::Command;
pub use snapshot::Snapshot;
pub use config::ResourcesBrokerConfig;
use crate::broker::event::Event;
use crate::broker::protocol::CommitReply;

use crate::repository::{Repository};
use crate::resource::generic::GenericMarshaller;

mod protocol;
mod snapshot;
mod config;
mod event;

type Subscriber = mpsc::Sender<Event>;

pub struct ResourcesBroker {
    repository: Repository<64>,
    subscribers: SlotMap<DefaultKey, Subscriber>,
    mailbox: Receiver<Command>,
}

impl ResourcesBroker {

    pub fn create(config: ResourcesBrokerConfig, marshaller: Box<dyn GenericMarshaller>) -> (mpsc::Sender<Command>, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel::<Command>(config.message_buffer_size);
        let handle = tokio::spawn(async move {
            let broker = ResourcesBroker {
                repository: Repository::new(marshaller),
                subscribers: Default::default(),
                mailbox: receiver,
            };
            match broker.run().await {
                Ok(_) => {
                    info!("ResourceBroker stopped gracefully.");
                }
                Err(_) => {
                    info!("ResourceBroker died unexpectedly!");
                }
            }
        });
        (sender, handle)
    }

    async fn run(mut self) -> Result<(), ()> {
        while let Some(command) = self.mailbox.recv().await {
            self.receive(command)?;
        }
        Ok(())
    }

    fn receive(&mut self, command: Command) -> Result<(), ()> {
        match command {
            Command::Subscribe { subscriber } => {
                let _key = self.subscribers.insert(subscriber);
            }
            Command::Snapshot { .. } => {}
            Command::Commit { resources, reply_to } => {
                for _resource in resources {
                    // self.storage.commit(resource).unwrap();
                }
                reply_to.send(CommitReply::Success)
                    .map_err(|_| {})?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use googletest::prelude::*;

    use crate::testkit;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test() -> Result<()> {

        let (mut broker, rm_handle) = ResourcesBroker::create(Default::default(), );

        let mut probe = testkit::probe::oneshot::<CommitReply>();

        broker.send(Command::Commit { resources: Vec::new(), reply_to: probe.sender() }).await?;

        probe.expect_message(|message| {
            verify_that!(message, eq(CommitReply::Success))
        }).await?;

        // resources_manager.send(Command::Subscribe {
        //     subscriber: probe.sender()
        // }).await?;

        // let mut probe = testkit::probe::oneshot::<Event>();

        // resources_manager.send(Command::Commit {
        //     resources: vec![Resource::ClusterConfiguration(cluster_configuration)],
        //     reply_to: probe.sender()
        // }).await?;

        Ok(())
    }

}
