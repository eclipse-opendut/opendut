use fs_err as fs;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;
use anyhow::{anyhow, Context};
use serde_json::json;
use tracing::debug;
use opendut_netbird_client_api::extension::LocalPeerStateExtension;
use opendut_util::pem;
use opendut_util::pem::ClientAuth;
use crate::common;
use crate::common::settings::netbird::NetbirdClientConfig;
use crate::common::util::create_file_and_ensure_it_can_only_be_read_or_modified_by_owner;
use crate::service::process_manager::{create_process_log_function, AsyncProcessId, AsyncProcessManager, AsyncProcessManagerExt, AsyncProcessManagerRef, OutputConfig, ProcessConfig, RestartPolicy};

const NETBIRD_CONFIG_DEFAULT_CLIENT_CERT_PATH: &str = "/etc/opendut/edgar/netbird/client.pem";
const NETBIRD_CONFIG_DEFAULT_CLIENT_KEY_PATH: &str = "/etc/opendut/edgar/netbird/client.key";

pub struct NetbirdProcess {
    process_manager: AsyncProcessManagerRef,
    process_id: AsyncProcessId,
}

impl NetbirdProcess {
    pub async fn spawn(config: NetbirdClientConfig) -> anyhow::Result<Self> {
        let process_manager = AsyncProcessManagerRef::new_shared();

        let netbird_config_file_path = common::settings::netbird_config_file_path();
        update_netbird_config(&config, netbird_config_file_path, NETBIRD_CONFIG_DEFAULT_CLIENT_CERT_PATH, NETBIRD_CONFIG_DEFAULT_CLIENT_KEY_PATH)?;

        let name = "netbird-client";
        let log_function = create_process_log_function!("opendut-netbird-client");

        let config = ProcessConfig::new(
            name,
            move || {
                let mut command = crate::setup::constants::netbird::command()
                    .expect("Unpacked NetBird executable path should be available.");

                command.arg("service")
                    .arg("run")
                    .arg("--config").arg(common::settings::netbird_config_file_path())
                    .arg("--daemon-addr=unix:///var/run/netbird.sock")
                    .arg("--log-level").arg(config.log_level.to_string())
                    .arg("--log-file=console")
                    .arg("--disable-profiles"); //not needed, since we manage the entire configuration and leads to errors when the NetBird process isn't running privileged

                command
            }
        )
        .with_restart_policy(RestartPolicy::Always)
        .with_restart_delay(Duration::from_secs(5))
        .with_output_config(OutputConfig::Capture);

        let process_id = AsyncProcessManager::spawn_process(process_manager.clone(), config, log_function).await?;

        Ok(Self {
            process_manager,
            process_id,
        })
    }

    pub async fn retrieve_remote_host(&self) -> anyhow::Result<IpAddr> {
        debug!("Determining remote IP address of host in NetBird VPN network.");
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;

        let status = client.full_status().await?;

        debug!("Netbird local peer state {:?}", status.local_peer_state);
        debug!("Netbird management state {:?}", status.management_state);
        debug!("Netbird signal state {:?}", status.signal_state);

        let host = status.local_peer_state
            .ok_or(anyhow!("NetBird Client did not return a local peer state. May not be logged in. Re-run `edgar setup` to fix this."))?
            .local_ip()?;

        Ok(IpAddr::from(host))
    }

    pub async fn terminate(self) -> anyhow::Result<()> {
        self.process_manager.lock().await
            .terminate(self.process_id).await
    }
}

fn update_netbird_config(
    config: &NetbirdClientConfig,
    netbird_config_file_path: impl AsRef<Path>,
    client_cert_path: impl AsRef<Path>,
    client_key_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let client_cert_path = client_cert_path.as_ref().to_str().context("Failed to convert client cert path to string.")?;
    let client_key_path = client_key_path.as_ref().to_str().context("Failed to convert client key path to string.")?;
    let netbird_config_content = if netbird_config_file_path.as_ref().exists() {
        fs::read_to_string(&netbird_config_file_path)?
    } else {
        String::from("{}")  // initially create empty NetBird config
    };
    create_parent_directory(&netbird_config_file_path, "NetBird configuration file")?;

    let fields = match &config.client_auth {
        ClientAuth::Enabled { certs, key } => {
            create_parent_directory(client_cert_path, "NetBird certificate file")?;
            create_parent_directory(client_key_path, "NetBird key file")?;

            // Netbird stores all other private keys in the config itself, evaluate if this is also possible here?
            let cert_string = pem::join_pem_objects(certs);
            let key_string = key.to_string();
            fs::write(client_cert_path, cert_string)?;
            create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(client_key_path)?;
            fs::write(client_key_path, key_string)?;
            vec![
                (config.config_path_client_cert.as_str(), json!(client_cert_path)),
                (config.config_path_client_key.as_str(), json!(client_key_path)),
            ]
        }
        ClientAuth::Disabled => {
            vec![
                (config.config_path_client_cert.as_str(), json!("")),
                (config.config_path_client_key.as_str(), json!("")),
            ]
        }
    };

    let json_config = add_fields_into_json_object_string(fields, &netbird_config_content, true)?;
    if netbird_config_content != json_config {
        fs::write(netbird_config_file_path, &json_config)?;
    }

    Ok(())
}

fn create_parent_directory(path: impl AsRef<Path>, name: &'static str) -> anyhow::Result<()> {
    let netbird_config_dir = path.as_ref()
        .parent()
        .context(format!("Failed to get parent directory of {}.", name))?;
    fs::create_dir_all(netbird_config_dir)
        .context(format!("Failed to create parent directory of {}.", name))
}


fn add_fields_into_json_object_string(
    fields: Vec<(&str, serde_json::Value)>,
    json: &str,
    pretty: bool,
) -> anyhow::Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(json)?;

    for (key, value) in fields.into_iter() {
        let pointer = normalize_to_pointer(key);
        if let Some(existing) = json.pointer_mut(&pointer) {
            *existing = value;
        } else {
            set_pointer_value_creating_objects(&mut json, &pointer, value)
                .context(format!("Failed to set NetBird config field `{}`.", key))?;
        }
    }
    let result = if pretty {
        serde_json::to_string_pretty(&json)
    } else {
        serde_json::to_string(&json)
    };
    result.context("Failed to serialize NetBird config with new fields.")
}

fn normalize_to_pointer(key: &str) -> String {
    if key.starts_with('/') {
        key.to_string()
    } else {
        format!("/{key}")
    }
}

fn set_pointer_value_creating_objects(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> anyhow::Result<()> {
    let mut tokens = pointer
        .split('/')
        .skip(1)
        .map(decode_json_pointer_token)
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }

    let last = tokens
        .pop()
        .context("JSON pointer did not contain a leaf token.")?;

    let mut current = root;
    for token in tokens {
        match current {
            serde_json::Value::Object(map) => {
                current = map
                    .entry(token)
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            }
            serde_json::Value::Array(array) => {
                let index: usize = token.parse()
                    .context("Failed to parse array index in JSON pointer.")?;
                current = array.get_mut(index)
                    .context("JSON pointer references an array index that does not exist.")?;
            }
            _ => return Err(anyhow!("JSON pointer traverses a non-container value.")),
        }
    }

    match current {
        serde_json::Value::Object(map) => {
            map.insert(last, value);
            Ok(())
        }
        serde_json::Value::Array(array) => {
            let index: usize = last.parse()
                .context("Failed to parse leaf array index in JSON pointer.")?;
            let destination = array.get_mut(index)
                .context("JSON pointer references a leaf array index that does not exist.")?;
            *destination = value;
            Ok(())
        }
        _ => Err(anyhow!("JSON pointer leaf parent is not an object or array.")),
    }
}

/// Because the characters '~' (%x7E) and '/' (%x2F) have special
/// meanings in JSON Pointer, '~' needs to be encoded as '~0' and '/'
/// needs to be encoded as '~1' when these characters appear in a
/// reference token.
/// See RFC6901: https://datatracker.ietf.org/doc/html/rfc6901#section-3
fn decode_json_pointer_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

#[cfg(test)]
mod tests {
    use assert_fs::fixture::PathChild;
    use assert_fs::TempDir;
    use super::*;
    use serde_json::json;
    use crate::common::settings;

    #[test]
    fn should_insert_field_into_json_object() -> anyhow::Result<()> {
        let config = sample_config();
        let field = vec![
            ("field1", json!("value1")),
            ("field2", json!(1234)),
        ];

        let result = add_fields_into_json_object_string(field, &config, false)?;

        assert_eq!(
            result,
            json!({
                "WgIface": "wt0",
                "WgIfaceMtu": 1280,
                "WgPort": 51820,
                "field1": "value1",
                "field2": 1234,
            }).to_string()
        );

        Ok(())
    }

    #[test]
    fn should_update_existing_nested_field_with_pointer() -> anyhow::Result<()> {
        let config = json!({
            "management": {
                "auth": {
                    "certPath": "old"
                }
            }
        })
        .to_string();

        let result = add_fields_into_json_object_string(
            vec![("/management/auth/certPath", json!("new"))],
            &config,
            false,
        )?;

        assert_eq!(
            result,
            json!({
                "management": {
                    "auth": {
                        "certPath": "new"
                    }
                }
            })
            .to_string()
        );

        Ok(())
    }

    #[test]
    fn should_create_missing_cascaded_structure_from_pointer() -> anyhow::Result<()> {
        let config = sample_config();

        let result = add_fields_into_json_object_string(
            vec![("/management/auth/certPath", json!("/tmp/cert.pem"))],
            &config,
            false,
        )?;

        assert_eq!(
            result,
            json!({
                "WgIface": "wt0",
                "WgIfaceMtu": 1280,
                "WgPort": 51820,
                "management": {
                    "auth": {
                        "certPath": "/tmp/cert.pem"
                    }
                }
            })
            .to_string()
        );

        Ok(())
    }

    #[test]
    fn should_decode_json_pointer_tokens() -> anyhow::Result<()> {
        let config = String::from("{}");

        let result = add_fields_into_json_object_string(
            vec![("/management~1auth/cert~0path", json!("value"))],
            &config,
            false,
        )?;

        assert_eq!(
            result,
            json!({
                "management/auth": {
                    "cert~path": "value"
                }
            })
            .to_string()
        );

        Ok(())
    }

    #[test]
    fn should_update_existing_array_element_with_pointer() -> anyhow::Result<()> {
        let config = json!({
            "items": [
                {}
            ]
        })
        .to_string();

        let result = add_fields_into_json_object_string(
            vec![("/items/0/name", json!("value"))],
            &config,
            false,
        )?;

        assert_eq!(
            result,
            json!({
                "items": [
                    {
                        "name": "value"
                    }
                ]
            })
            .to_string()
        );

        Ok(())
    }


    /// Snippet with similar structure to NetBird config.json
    fn sample_config() -> String {
        json!({
            "WgIface": "wt0",
            "WgIfaceMtu": 1280,
            "WgPort": 51820,
        })
            .to_string()
    }

    #[test]
    fn should_add_client_config_field() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let netbird_config_path = temp.child("netbird-config.json");
        let client_cert_path = temp.child("client.pem");
        let client_key_path = temp.child("client.key");
        let cert = include_str!("../../../../resources/development/tls/insecure-development-ca.pem");
        let key = include_str!("../../../../resources/development/tls/insecure-development-ca.key");

        let config = config::Config::builder()
            .set_override(pem::config_keys::VPN_NETBIRD_CLIENT_TLS_CLIENT_AUTH.enabled, true)?
            .set_override(pem::config_keys::VPN_NETBIRD_CLIENT_TLS_CLIENT_AUTH.certificate, cert.to_string())?
            .set_override(pem::config_keys::VPN_NETBIRD_CLIENT_TLS_CLIENT_AUTH.key, key.to_string())?
            .set_override(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled, false)?
            .set_override(settings::key::netbird::client::config::keys::mtls::certificate, "MgmtClientCert/CertPath")?
            .set_override(settings::key::netbird::client::config::keys::mtls::key, "MgmtClientCert/KeyPath")?
            .build()?;

        let loaded_config = settings::load_with_overrides(config)?;
        let netbird_config = NetbirdClientConfig::load_from_config(&loaded_config)?;
        matches!(netbird_config.client_auth, ClientAuth::Enabled { .. });

        update_netbird_config(&netbird_config, netbird_config_path.path(), client_cert_path.path(), client_key_path.path())?;

        assert!(netbird_config_path.exists(), "NetBird config should exist.");
        assert!(client_cert_path.exists(), "NetBird client certificate should exist.");
        assert!(client_key_path.exists(), "NetBird client key should exist.");

        let mut config_content = fs::read_to_string(netbird_config_path)?;
        config_content.retain(|c| !c.is_whitespace());
        assert_eq!(
            config_content,
            json!({
                "MgmtClientCert": {
                    "CertPath": client_cert_path.path().display().to_string(),
                    "KeyPath": client_key_path.path().display().to_string(),
                }
            }).to_string()
        );

        Ok(())
    }
}
