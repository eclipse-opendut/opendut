use std::collections::HashMap;
use std::net::Ipv4Addr;

use anyhow::anyhow;
use phf::phf_map;
use serde::{Deserialize, Deserializer};

use crate::core::docker::command::DockerCommand;
use crate::core::util::consume_output;

fn ip_address_from_str<'de, D>(deserializer: D) -> Result<Ipv4Addr, D::Error>
    where D: Deserializer<'de>
{
    let string = String::deserialize(deserializer)?;
    let ip_string = string
        .trim_matches('\"')
        .trim_end_matches("/24")
        .to_string();
    ip_string.parse::<Ipv4Addr>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
struct ContainerAddress {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "EndpointID")]
    _endpoint_id: String,
    #[serde(rename = "MacAddress")]
    _mac_address: String,
    #[serde(rename = "IPv4Address", deserialize_with = "ip_address_from_str")]
    ipv4address: Ipv4Addr,
    #[serde(rename = "IPv6Address")]
    _ipv6address: String,
}

enum DockerHostnames {
    Carl,
    Keycloak,
    NetbirdManagement,
    NetbirdDashboard,
    Firefox,
}

impl DockerHostnames {
    fn as_str(&self) -> &'static str {
        match self {
            DockerHostnames::Carl => "carl.opendut.local",
            DockerHostnames::Keycloak => "auth.opendut.local",
            DockerHostnames::NetbirdManagement => "netbird-api.opendut.local",
            DockerHostnames::NetbirdDashboard => "netbird.opendut.local",
            DockerHostnames::Firefox => "firefox",
        }
    }
}
static CONTAINER_NAME_MAP: phf::Map<&'static str, DockerHostnames> = phf_map! {
    "opendut-firefox" => DockerHostnames::Firefox,
    "opendut-keycloak" => DockerHostnames::Keycloak,
    "opendut-carl" => DockerHostnames::Carl,
    "opendut-netbird-management" => DockerHostnames::NetbirdManagement,
    "opendut-netbird-dashboard" => DockerHostnames::NetbirdDashboard,
};

pub(crate) fn docker_inspect_network() -> crate::Result {
    let output = DockerCommand::new()
        .arg("network")
        .arg("inspect")
        .arg("opendut_local")
        .arg("--format")
        .arg("'{{json .Containers}}'")
        .debug_log_executed_command()
        .output();

    let stdout = match consume_output(output) {
        Err(error) => {
            return Err(anyhow!("Failed to inspect docker network: {:?}", error));
        }
        Ok(stdout) => { stdout.trim_matches('\'').to_string() }
    };

    let opendut_container_address_map: HashMap<String, ContainerAddress> = match
        serde_json::from_str(&stdout) {
        Ok(map) => { map }
        Err(error) => {
            return Err(anyhow!("Failed to parse json: {:?}", error));
        }
    };
    let mut sorted_addresses: Vec<(&String, &ContainerAddress)> = opendut_container_address_map.iter().collect();
    sorted_addresses
        .sort_by(|a, b| a.1.ipv4address.cmp(&b.1.ipv4address));

    let message = "OpenDuT docker network 'docker network inspect opendut_local'";
    println!("# BEGIN {message}");
    for (_key, value) in &sorted_addresses {
        let ip_address = value.ipv4address.to_string();
        let given_hostname = value.name.clone();
        let hostname = if CONTAINER_NAME_MAP.contains_key(given_hostname.as_str()) {
            CONTAINER_NAME_MAP.get(given_hostname.as_str()).unwrap().as_str().to_string()
        } else {
            given_hostname
        };

        let padding = std::cmp::max(0, 20 - ip_address.clone().len());
        let whitespace = " ".repeat(padding);
        let padded_ip_address = ip_address.clone() + &whitespace;
        println!("{padded_ip_address}  {hostname}");
    }
    println!("# END {message}");
    Ok(())
}
