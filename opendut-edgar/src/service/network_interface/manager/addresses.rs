use std::net::IpAddr;
use futures::TryStreamExt;
use netlink_packet_route::address::{AddressAttribute, AddressMessage};
use netlink_packet_route::AddressFamily;
use opendut_types::util::net::NetworkInterfaceName;
use crate::service::network_interface::manager::interface::Interface;
use crate::service::network_interface::manager::{Error, NetworkInterfaceManager};

impl NetworkInterfaceManager {
    pub async fn add_address(&self, interface: &Interface, address: IpAddr, prefix_len: u8) -> Result<Interface, Error> {
        let interface = self.try_find_interface(&interface.name).await?;
        self.handle.address().add(interface.index, address, prefix_len)
            .execute()
            .await
            .map_err(|error| Error::ModificationFailure { name: interface.name.clone(), cause: format!("Failed to add ip address. {}", error) })?;
        Ok(interface)
    }

    pub async fn delete_address(&self, interface: &Interface, address: IpAddr, prefix_len: u8) -> Result<Interface, Error> {
        let address_family = match address {
            IpAddr::V4(_) => {
                AddressFamily::Inet
            }
            IpAddr::V6(_) => {
                AddressFamily::Inet6
            }
        };
        let mut address_message = AddressMessage::default();
        address_message.header.index = interface.index;
        address_message.header.prefix_len = prefix_len;
        address_message.header.family = address_family;
        address_message.attributes = vec![AddressAttribute::Address(address)];

        self.handle.address().del(address_message.clone()).execute()
            .await
            .map_err(|error| Error::ModificationFailure { name: interface.name.clone(), cause: format!("Failed to delete ip address. {}", error) })?;
        let interface = self.try_find_interface(&interface.name).await?;

        Ok(interface)
    }


    pub async fn flush_addresses(&self, name: &NetworkInterfaceName) -> Result<(), Error> {
        let link = self.handle
            .link().get().match_name(name.name()).execute()
            .try_next().await
            .map_err(|cause| Error::ListInterfaces { cause: cause.into() })?
            .ok_or(Error::InterfaceNotFound { name: name.clone() })?;

        let addresses = self.handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute()
            .try_collect::<Vec<AddressMessage>>().await
            .map_err(|cause| Error::ListAddresses { cause: cause.into() } )?;
        for address in addresses {
            self.handle.address().del(address).execute().await
                .map_err(|cause| Error::DeleteAddress { name: name.clone(), cause: cause.into() })?;
        }

        Ok(())
    }

    pub async fn get_addresses(&self) -> Result<(), Error> {
        let links = self.handle
            .link()
            .get()
            .execute()
            .try_collect::<Vec<_>>()
            .await
            .map_err(|cause| Error::ListInterfaces { cause: cause.into() })?;

        for link in links {
            let mut address_messages = self.handle
                .address()
                .get()
                .set_link_index_filter(link.header.index)
                .execute()
                .try_collect::<Vec<_>>()
                .await
                .map_err(|cause| Error::ListInterfaces { cause: cause.into() })?;
            for address_message in address_messages.iter_mut() {
                let ips = address_message.attributes.iter().filter_map(|attribute| {
                    if let AddressAttribute::Address(ip) = attribute {
                        Some(*ip)
                    } else {
                        None
                    }
                }).collect::<Vec<IpAddr>>();
                println!("{address_message:?}");
                for ip in ips.clone() {
                    match ip {
                        IpAddr::V4(_ip) => {

                        }
                        IpAddr::V6(_) => {}
                    }
                }
                println!("{ips:?}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use opendut_types::util::net::NetworkInterfaceName;
    use crate::service::network_interface::manager::NetworkInterfaceManager;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[tokio::test]
    async fn test_create_dummy_interface() -> anyhow::Result<()> {
        let (connection, handle, _) = rtnetlink::new_connection().unwrap();
        tokio::spawn(connection);

        let manager = NetworkInterfaceManager { handle };

        let dummy = NetworkInterfaceName::try_from("dummy123".to_string())?;
        let address = IpAddr::V4(Ipv4Addr::new(123, 1, 1, 1));

        let interface = manager.create_dummy_ipv4_interface(&dummy).await?;
        let interface = manager.add_address(&interface, address, 24).await?;
        let interface = manager.delete_address(&interface, address, 24)
            .await.inspect_err(|cause| println!("Failed to delete address: {cause}"))?;

        manager.delete_interface(&interface).await?;

        Ok(())
    }
}