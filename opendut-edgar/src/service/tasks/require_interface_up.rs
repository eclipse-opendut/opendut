use anyhow::anyhow;
use async_trait::async_trait;
use rtnetlink::packet_route::link::LinkFlags;
use opendut_model::util::net::NetworkInterfaceName;
use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;


pub struct RequireInterfaceUp {
    pub interface: NetworkInterfaceName,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

#[async_trait]
impl Task for RequireInterfaceUp {
    fn description(&self) -> String {
        format!("Require interface '{}' is up", self.interface)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.interface).await?;

        match interface {
            Some(interface) => {
                let interface_is_up = interface.link_flags.contains(LinkFlags::Up);

                if interface_is_up {
                    Ok(TaskStateFulfilled::Yes)
                } else {
                    Ok(TaskStateFulfilled::No)
                }
            }
            None => Ok(TaskStateFulfilled::No),
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        Err(anyhow!("Interface check did not return that interface exists and is up!")) //always fail, if we end up in `make_present()`
    }
}

#[async_trait]
impl TaskAbsent for RequireInterfaceUp {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }
    async fn make_absent(&self) -> anyhow::Result<Success> {
        Ok(Success::default())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use opendut_model::util::net::NetworkInterfaceName;
    use crate::service::network_interface::manager::NetworkInterfaceManager;


    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn should_detect_interface_is_up() -> anyhow::Result<()> {
        let network_interface_manager = NetworkInterfaceManager::create()
            .expect("Failed to spawn NetworkInterfaceManager.");

        let existing_interface = {
            let interface = NetworkInterfaceName::try_from("lo")?;

            let result = network_interface_manager.find_interface(&interface).await?;
            assert!(result.is_some());

            interface
        };

        let testee = RequireInterfaceUp {
            interface: existing_interface,
            network_interface_manager,
        };

        let result = testee.check_present().await?;
        assert_eq!(result, TaskStateFulfilled::Yes);

        Ok(())
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn should_fail_when_the_interface_is_not_up() -> anyhow::Result<()> {
        let network_interface_manager = NetworkInterfaceManager::create()
            .expect("Failed to spawn NetworkInterfaceManager.");

        let non_existing_interface = {
            let interface = NetworkInterfaceName::try_from("non_existing")?;

            let result = network_interface_manager.find_interface(&interface).await?;
            assert!(result.is_none());

            interface
        };

        let testee = RequireInterfaceUp {
            interface: non_existing_interface,
            network_interface_manager,
        };

        let result = testee.check_present().await?;
        assert_eq!(result, TaskStateFulfilled::No);

        let result = testee.make_present().await;
        assert!(result.is_err());

        Ok(())
    }
}
