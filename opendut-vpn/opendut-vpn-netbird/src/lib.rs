use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Url;
use tracing::{debug, error, info, trace, warn};

pub use netbird::Token as NetbirdToken;
use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_vpn::{CreateClusterError, CreatePeerError, CreateVpnPeerConfigurationError, DeleteClusterError, DeletePeerError, VpnManagementClient};

use crate::client::{Client, DefaultClient};
use crate::netbird::error::{CreateClientError, CreateSetupKeyError, GetGroupError, GetRulesError, RequestError};
use crate::netbird::GroupName;

mod client;
mod routes;
mod netbird;

type Inner = Box<dyn Client + Send + Sync>;

pub struct NetbirdManagementClientConfiguration {
    pub management_url: Url,
    pub authentication_token: Option<NetbirdToken>,
    pub ca: Option<PathBuf>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>
}

pub struct NetbirdManagementClient {
    management_url: Url,
    inner: Inner,
}

impl NetbirdManagementClient {

    pub fn create(configuration: NetbirdManagementClientConfiguration) -> Result<Self, CreateClientError> {
        let management_url = configuration.management_url;
        let management_ca_path = configuration.ca
            .ok_or_else(|| CreateClientError::InstantiationFailure { cause: String::from("No ca certificate provided.") })?;
        let management_ca = {
            let mut file = File::open(management_ca_path)
                .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to open ca certificate:\n  {cause}") })?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to read ca certificate:\n  {cause}") })?;
            buffer
        };
        let inner = Box::new(DefaultClient::create(
            Clone::clone(&management_url),
            Some(management_ca.as_slice()),
            configuration.authentication_token,
            None,
            configuration.timeout,
            configuration.retries
        )?);
        Ok(Self {
            management_url,
            inner
        })
    }
}

#[async_trait]
impl VpnManagementClient for NetbirdManagementClient {

    async fn create_cluster(&self, cluster_id: ClusterId, peers: &[PeerId]) -> Result<(), CreateClusterError> {

        match self.delete_cluster(cluster_id).await {
            Ok(_) => debug!("Deleted a previous cluster with ID <{cluster_id}> before creating the new cluster."),
            Err(cause) => match cause {
                DeleteClusterError::NotFound { cluster_id, message } => trace!("Did not need to delete a previous cluster with ID <{cluster_id}> before creating the new cluster. ({message})"),
                DeleteClusterError::DeletionFailure { cluster_id, error } => {
                    return Err(CreateClusterError::CreationFailure { cluster_id, error: anyhow!("Failure while deleting a previous cluster with ID <{cluster_id}> before creating the new cluster: {error}").into() });
                }
            }
        };

        let netbird_peers: Vec<netbird::PeerId> = {
            let mut netbird_peers = vec![];
            for peer_id in peers {
                let group = self.inner.get_netbird_group(&(*peer_id).into()).await
                    .map_err(|error| CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: error.into() })?;
                let peer = group.peers.into_iter().next()
                    .ok_or(CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: anyhow!("Self-Group does not contain expected peer!").into() })?;
                netbird_peers.push(peer.id);
            }
            netbird_peers
        };

        let group = self.inner.create_netbird_group(cluster_id.into(), netbird_peers).await
            .map_err(|error| CreateClusterError::CreationFailure { cluster_id, error: error.into() })?;

        self.inner.create_netbird_self_access_control_rule(group, cluster_id.into()).await
            .map_err(|error| CreateClusterError::AccessControlRuleCreationFailure { cluster_id, error: error.into() })?;

        Ok(())
    }

    async fn delete_cluster(&self, cluster_id: ClusterId) -> Result<(), DeleteClusterError> {
        let rule_name = netbird::RuleName::Cluster(cluster_id);
        match self.inner.get_netbird_rule(&rule_name).await {
            Ok(rule) => {
                match self.inner.delete_netbird_rule(&rule.id).await {
                    Ok(_) => debug!("Deleted NetBird rule with name '{}' and NetBird Rule ID '{}'.", rule.name, rule.id.0),
                    Err(cause) => return match cause {
                        RequestError::IllegalStatus(error) => {
                            if let Some(http::StatusCode::NOT_FOUND) = error.status() {
                                Err(DeleteClusterError::NotFound { cluster_id, message: format!("Received '404 Not Found' when deleting rule for cluster <{cluster_id}> with NetBird rule ID <{netbird_rule}>.", netbird_rule = rule.id.0) })
                            } else {
                                Err(DeleteClusterError::DeletionFailure { cluster_id, error: error.into() })
                            }
                        }
                        RequestError::IllegalRequest(error, code) => {
                            Err(DeleteClusterError::DeletionFailure {
                                cluster_id,
                                error: anyhow!("Received status code '{code}' when deleting cluster <{cluster_id}> with NetBird rule ID <{netbird_rule}>:\n  {error}", code=code, cluster_id=cluster_id, netbird_rule=rule.id.0, error=error).into(),
                            })
                        }
                        other => Err(DeleteClusterError::DeletionFailure { cluster_id, error: other.into() }),
                    }
                }
            }
            Err(GetRulesError::RuleNotFound { .. }) => {
                // No rule found, so no need to delete it.
            }
            Err(cause) => {
                return Err(DeleteClusterError::DeletionFailure { cluster_id, error: anyhow!("Failed to get cluster rule '{rule_name}' to be deleted.\n {cause}").into() });
            }
        };

        let group_name = netbird::GroupName::from(cluster_id);
        match self.inner.get_netbird_group(&group_name).await {
            Ok(group) => {
                match self.inner.delete_netbird_group(&group.id).await {
                    Ok(_) => {
                        debug!("Deleted NetBird group with name '{}' and NetBird Group ID '{}'.", group.name, group.id.0) ;
                        Ok(())
                    },
                    Err(cause) => match cause {
                        RequestError::IllegalStatus(error) => {
                            if let Some(http::StatusCode::NOT_FOUND) = error.status() {
                                Err(DeleteClusterError::NotFound { cluster_id, message: format!("Received '404 Not Found' when deleting group for cluster <{cluster_id}> with NetBird group ID <{netbird_group}>.", netbird_group = group.id.0) })
                            } else {
                                Err(DeleteClusterError::DeletionFailure { cluster_id, error: error.into() }) //TODO logging of this doesn't show the HTTP body, making e.g. 400 Bad Request errors difficult to debug
                            }
                        }
                        other => Err(DeleteClusterError::DeletionFailure { cluster_id, error: other.into() }),
                    }
                }
            }
            Err(GetGroupError::GroupNotFound { .. }) => {
                // No group found, so no need to delete it.
                Ok(())
            }
            Err(cause) => {
                Err(DeleteClusterError::DeletionFailure { cluster_id, error: anyhow!("Failed to get cluster group '{group_name}' to be deleted.\n {cause}").into() })
            }
        }
    }

    async fn create_peer(&self, peer_id: PeerId) -> Result<(), CreatePeerError> {
        let peers = vec![]; //Peer self-group does not have peers
        self.inner.create_netbird_group(peer_id.into(), peers).await
            .map_err(|error| CreatePeerError::CreationFailure { peer_id, error: error.into() })?;
        Ok(())
    }

    async fn delete_peer(&self, peer_id: PeerId) -> Result<(), DeletePeerError> {
        let self_group = self.inner.get_netbird_group(&netbird::GroupName::from(peer_id)).await
            .map_err(|error| DeletePeerError::ResolutionFailure { peer_id, error: error.into() })?;

        if let Some(peer_info) = self_group.peers.first() {
            self.inner.delete_netbird_peer(&peer_info.id)
                .await
                .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;
        }

        self.inner.delete_netbird_group(&self_group.id)
            .await
            .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;

        Ok(())
    }

    async fn generate_vpn_peer_configuration(&self, peer_id: PeerId) -> Result<VpnPeerConfiguration, CreateVpnPeerConfigurationError> {

        debug!("Generating vpn configuration for peer <{peer_id}>.");

        let self_group_name = GroupName::Peer(peer_id);

        match self.inner.get_netbird_group(&self_group_name).await {
            Ok(group) => {
                debug!("Deleting self group '{self_group_name}' of peer <{peer_id}>.");
                self.inner.delete_netbird_group(&group.id).await
                    .map_err(|error| {
                        error!("Failed to generate vpn configuration for peer <{peer_id}>, du to communication issues when deleting the peer's self group '{self_group_name}'!");
                        CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: Box::new(error) }
                    })?;
                info!("Successfully deleted self group '{self_group_name}' of peer <{peer_id}>.");
            }
            Err(GetGroupError::GroupNotFound { .. }) => {
                warn!("There is no self group '{self_group_name}' for peer <{peer_id}> to delete. This might indicate an invalid state!")
            }
            Err(error @ GetGroupError::MultipleGroupsFound { .. }) => {
                error!("Failed to generate vpn configuration for peer <{peer_id}>, because there are multiple groups with the same name '{self_group_name}'! This is an invalid state!");
                Err(CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: Box::new(error) })?;
            }
            Err(error @ GetGroupError::RequestFailure { .. }) => {
                error!("Failed to generate vpn configuration for peer <{peer_id}>, due to communication issues when trying to look up the peer's self group!");
                Err(CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: Box::new(error) })?;
            }
        };

        debug!("Re-creating self group '{self_group_name}' for peer <{peer_id}>.");

        self.inner.create_netbird_group(Clone::clone(&self_group_name), Vec::new()).await
            .map_err(|error| {
                error!("Failed to generate vpn configuration for peer <{peer_id}>, due to communication issues when re-creating the peer's self group '{self_group_name}'!");
                CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: Box::new(error) }
            })?;

        debug!("Requesting setup key for peer <{peer_id}>.");

        let setup_key = self.inner.generate_netbird_setup_key(peer_id).await
            .map_err(|error| match error {
                CreateSetupKeyError::PeerGroupNotFound { cause: error, .. } => {
                    error!("Failed to generate vpn configuration for peer <{peer_id}>, because the peer's self group could not be found!");
                    CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: error.into() }
                }
                CreateSetupKeyError::RequestFailure { .. } => {
                    error!("Failed to generate vpn configuration for peer <{peer_id}>, du to communication issues when requesting the new setup key!");
                    CreateVpnPeerConfigurationError::CreationFailure { peer_id, error: error.into() }
                }
            })?;

        debug!("Successfully generated vpn configuration for peer <{peer_id}>.");

        Ok(VpnPeerConfiguration::Netbird {
            management_url: Clone::clone(&self.management_url),
            setup_key: opendut_types::vpn::netbird::SetupKey::from(setup_key.key),
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use async_trait::async_trait;
    use googletest::prelude::*;
    use reqwest::Url;
    use time::OffsetDateTime;
    use uuid::uuid;

    use opendut_types::cluster::ClusterId;
    use opendut_types::peer::PeerId;
    use opendut_types::vpn;
    use opendut_types::vpn::VpnPeerConfiguration;
    use opendut_vpn::VpnManagementClient;

    use crate::{netbird, NetbirdManagementClient};
    use crate::client::Client;
    use crate::netbird::error::{CreateSetupKeyError, GetGroupError, GetRulesError, RequestError};
    use crate::netbird::GroupPeerInfo;

    #[tokio::test]
    async fn A_NetbirdManagementClient_should_create_a_cluster_by_creating_a_netbird_group_containing_all_peers_of_the_cluster() -> Result<()> {

        let cluster_id = ClusterId::from(uuid!("6a6510a9-031b-4834-a4f7-454cc401fe13"));
        let peer_a_id = PeerId::from(uuid!("d61bed7b-2fec-4a5b-a937-d6a791cb5ff9"));
        let peer_b_id = PeerId::from(uuid!("267ca2c0-942e-4780-857e-c153ddcf2328"));
        let peer_a_group_name = netbird::GroupName::from(peer_a_id);
        let peer_b_group_name = netbird::GroupName::from(peer_b_id);
        let cluster_group_name = netbird::GroupName::from(cluster_id);
        let cluster_group_id = netbird::GroupId::from("cluster-group");
        let peer_a_group = netbird::Group {
            id: netbird::GroupId::from("peer-a-group"),
            name: Clone::clone(&peer_a_group_name),
            peers_count: 0,
            peers: vec![GroupPeerInfo { id: netbird::PeerId::from("peer-a"), name: String::from("peer-a")}],
        };
        let peer_b_group = netbird::Group {
            id: netbird::GroupId::from("peer-b-group"),
            name: Clone::clone(&peer_b_group_name),
            peers_count: 0,
            peers: vec![GroupPeerInfo { id: netbird::PeerId::from("peer-b"), name: String::from("peer-b")}],
        };
        let cluster_group = netbird::Group {
            id: Clone::clone(&cluster_group_id),
            name: Clone::clone(&peer_b_group_name),
            peers_count: 0,
            peers: vec![],
        };

        let fixture = Fixture::setup(|mock_client| {
            mock_client.expect_get_netbird_rule()
                .returning(|rule_name| Err(GetRulesError::RuleNotFound { rule_name: rule_name.to_owned() }));
            mock_client.expect_get_netbird_group()
                .returning({
                    let cluster_group_name = Clone::clone(&cluster_group_name);
                    let cluster_group = Clone::clone(&cluster_group);
                    move |group_name| {
                        if group_name == &peer_a_group_name {
                            Ok(Clone::clone(&peer_a_group))
                        }
                        else if group_name == &peer_b_group_name {
                            Ok(Clone::clone(&peer_b_group))
                        }
                        else if group_name == &cluster_group_name {
                            Ok(Clone::clone(&cluster_group))
                        }
                        else {
                            Err(GetGroupError::GroupNotFound { group_name: group_name.to_owned() })
                        }
                    }
                });
            mock_client.expect_delete_netbird_group()
                .times(1)
                .withf(move |actual_group_id| *actual_group_id == cluster_group_id)
                .returning(|_| Ok(()));
            mock_client.expect_create_netbird_group()
                .times(1)
                .withf(move |actual_cluster_group_name, _actual_peers| {
                    actual_cluster_group_name == &cluster_group_name // TODO: Check that _actual_peers contains all relevant peers
                })
                .returning({
                    let cluster_group = Clone::clone(&cluster_group);
                    move |_, _| Ok(Clone::clone(&cluster_group))
                });
            mock_client.expect_create_netbird_self_access_control_rule()
                .times(1)
                .withf(move |actual_group, _rule_name| actual_group == &cluster_group)
                .returning(|_, _| Ok(()));
        });

        assert_that!(fixture.testee.create_cluster(cluster_id, &vec![peer_a_id, peer_b_id]).await, ok(anything()));

        Ok(())
    }

    #[tokio::test]
    async fn A_NetbirdManagementClient_should_delete_the_self_group_of_a_peer_when_creating_a_peer_configuration() -> Result<()> {

        let peer_id = PeerId::from(uuid!("d61bed7b-2fec-4a5b-a937-d6a791cb5ff9"));
        let peer_self_group_name = netbird::GroupName::from(peer_id);
        let peer_self_group_id = netbird::GroupId::from("peer-self-group");
        let peer_self_group = netbird::Group {
            id: Clone::clone(&peer_self_group_id),
            name: Clone::clone(&peer_self_group_name),
            peers_count: 0,
            peers: vec![],
        };
        let setup_key = netbird::SetupKey {
            id: String::from("some-id"),
            key: uuid!("4626c02a-bee7-4468-91c3-73c47fd0116c"),
            name: netbird::setup_key_name_format(peer_id),
            expires: netbird::SetupKetTimeStamp { inner: OffsetDateTime::now_utc() },
            r#type: netbird::SetupKeyType::OneOff,
            valid: true,
            revoked: false,
            used_times: 0,
            last_used: netbird::SetupKetTimeStamp { inner: OffsetDateTime::now_utc() },
            state: netbird::SetupKeyState::Valid,
            auto_groups: vec![String::from("ch8i4ug6lnn4g9hqv7m0")],
            updated_at: netbird::SetupKetTimeStamp { inner: OffsetDateTime::now_utc() },
            usage_limit: 0,
        };

        let fixture = Fixture::setup(|mock_client| {
            mock_client.expect_get_netbird_group()
                .returning({
                    let peer_self_group_name = Clone::clone(&peer_self_group_name);
                    let peer_self_group = Clone::clone(&peer_self_group);
                    move |group_name| {
                        if group_name == &peer_self_group_name  {
                            Ok(Clone::clone(&peer_self_group))
                        }
                        else {
                            Err(GetGroupError::GroupNotFound { group_name: group_name.to_owned() })
                        }
                    }
                });
            mock_client.expect_delete_netbird_group()
                .times(1)
                .withf(move |actual_group_id| *actual_group_id == peer_self_group_id)
                .returning(|_| Ok(()));
            mock_client.expect_create_netbird_group()
                .times(1)
                .withf(move |actual_group_name, peers| {
                    *actual_group_name == peer_self_group_name && peers.is_empty()
                })
                .returning(move |_, _| Ok(Clone::clone(&peer_self_group)));
            mock_client.expect_generate_netbird_setup_key()
                .times(1)
                .withf(move |actual_peer_id| actual_peer_id.0 == peer_id.0)
                .returning({
                    let setup_key = Clone::clone(&setup_key);
                    move |_| {
                        Ok(Clone::clone(&setup_key))
                    }
                });
        });

        assert_that!(fixture.testee.generate_vpn_peer_configuration(peer_id).await,
            ok(matches_pattern!(VpnPeerConfiguration::Netbird {
                management_url: eq(Url::parse("https://localhost/api/").unwrap()),
                setup_key: matches_pattern!(vpn::netbird::SetupKey {
                    uuid: eq(setup_key.key)
                }),
            })
        ));

        Ok(())
    }

    struct Fixture {
        testee: NetbirdManagementClient,
    }

    impl Fixture {
        fn setup<F>(setup: F) -> Fixture
        where
            F: FnOnce(&mut MockMockClient) -> ()
        {
            let mut mock_client = MockMockClient::new();
            setup(&mut mock_client);
            let testee = NetbirdManagementClient {
                management_url: Url::parse("https://localhost/api/").unwrap(),
                inner: Box::new(mock_client),
            };

            Fixture {
                testee,
            }
        }
    }

    mockall::mock! {
        pub MockClient {}
        #[async_trait]
        impl Client for MockClient {
            async fn create_netbird_group(&self, name: netbird::GroupName, peers: Vec<netbird::PeerId>) -> std::result::Result<netbird::Group, RequestError>;
            async fn get_netbird_group(&self, group_name: &netbird::GroupName) -> std::result::Result<netbird::Group, GetGroupError>;
            async fn delete_netbird_group(&self, group_id: &netbird::GroupId) -> std::result::Result<(), RequestError>;
            async fn get_netbird_peer(&self, peer_id: &netbird::PeerId) -> std::result::Result<netbird::Peer, RequestError>;
            async fn delete_netbird_peer(&self, peer_id: &netbird::PeerId) -> std::result::Result<(), RequestError>;
            async fn create_netbird_self_access_control_rule(&self, group: netbird::Group, rule_name: netbird::RuleName) -> std::result::Result<(), RequestError>;
            async fn get_netbird_rule(&self, rule_name: &netbird::RuleName) -> std::result::Result<netbird::Rule, GetRulesError>;
            async fn delete_netbird_rule(&self, rule_id: &netbird::RuleId) -> std::result::Result<(), RequestError>;
            async fn generate_netbird_setup_key(&self, peer_id: PeerId) -> std::result::Result<netbird::SetupKey, CreateSetupKeyError>;
        }
    }
}
