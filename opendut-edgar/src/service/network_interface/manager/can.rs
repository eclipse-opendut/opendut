use std::process::Command;
use serde::{Deserialize, Deserializer};
use opendut_model::util::net::NetworkInterfaceName;
use crate::service::network_interface::manager::interface::NetlinkInterfaceKind;
use crate::service::network_interface::manager::NetworkInterfaceManager;


#[derive(Deserialize)]
struct LinkInfo {
    info_kind: String,
    info_data: Option<LinkInfoData>,
}

fn as_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where D: Deserializer<'de>
{
    let value = String::deserialize(deserializer)?;
    let float = value.parse::<f32>()
        .map_err(serde::de::Error::custom)?;
    Ok(float)
}

#[derive(Clone, Copy, Deserialize, PartialEq, Debug)]
pub struct BitTiming {
    pub bitrate: u32,
    #[serde(deserialize_with = "as_f32")]
    pub sample_point: f32,
}
#[derive(Deserialize)]
struct LinkInfoData {
    #[serde(rename = "ctrlmode")]
    control_mode: Option<Vec<String>>,
    #[serde(rename = "bittiming")]
    bit_timing: BitTiming,
    #[serde(rename = "data_bittiming")]
    can_fd_bit_timing: Option<BitTiming>
}
#[derive(Deserialize)]
struct IpLinkShowOutputForCan {
    #[serde(rename = "ifindex")]
    _interface_index: usize,
    #[serde(rename = "ifname")]
    _interface_name: String,
    #[serde(rename = "operstate")]
    _operational_state: String,
    #[serde(rename = "linkinfo")]
    link_info: LinkInfo
}

const IF_CONTROL_MODE_CAN_FD: &str = "FD";

const IP_LINK_INFO_KIND_CAN: &str = "can";

#[derive(Debug, PartialEq)]
pub struct CanInterfaceConfiguration {
    pub bit_timing: BitTiming,
    pub fd: CanFD,
}

#[derive(Debug, PartialEq)]
pub enum CanFD {
    Enabled(BitTiming),
    Disabled,
}

impl NetworkInterfaceManager {
    pub async fn detect_can_device_configuration(&self, name: NetworkInterfaceName) -> anyhow::Result<CanInterfaceConfiguration> {
        let interface = self.find_interface(&name).await?
            .ok_or_else(|| anyhow::anyhow!("CAN interface '{}' not found", name) )?;

        match interface.kind {
            NetlinkInterfaceKind::Can(_) => {}
            _ => {
                return Err(anyhow::Error::msg(format!("Interface '{}' is not a CAN interface.", name.name())));
            }
        }
        let can_configuration = ip_link_show(name)?;

        Ok(can_configuration)
    }
}

fn ip_link_show(name: NetworkInterfaceName) -> anyhow::Result<CanInterfaceConfiguration> {
    let command = Command::new("ip")
        .args(["link", "-json", "-details", "show", "dev", &name.name()])
        .output()?;

    if !command.status.success() {
        return Err(anyhow::Error::msg(format!("Failed to execute 'ip link show' command for device '{}'.", name.name())));
    }
    let output_str = String::from_utf8_lossy(&command.stdout);
    let can_config = extract_ip_link_can_configuration(name, &output_str)?;
    Ok(can_config)
}

fn extract_ip_link_can_configuration(name: NetworkInterfaceName, ip_link_show: &str) -> anyhow::Result<CanInterfaceConfiguration> {
    let ip_link_list: Vec<IpLinkShowOutputForCan> = serde_json::from_str(ip_link_show)?;
    match ip_link_list.into_iter().next() {
        None => {
            Err(anyhow::Error::msg(format!("No output from 'ip link show' command for device '{}'.", name.name())))
        }
        Some(ip_link) => {
            let can_configuration = convert_link_info_to_can_configuration(name.clone(), &ip_link.link_info)?;
            Ok(can_configuration)
        }
    }
}

fn convert_link_info_to_can_configuration(name: NetworkInterfaceName, link_info: &LinkInfo) -> anyhow::Result<CanInterfaceConfiguration> {
    if link_info.info_kind != IP_LINK_INFO_KIND_CAN {
        return Err(anyhow::Error::msg(format!("Interface '{}' is not a CAN interface according to 'ip link show' output.", name.name())));
    }

    let info_data = link_info.info_data.as_ref().ok_or_else(|| anyhow::Error::msg("Missing info_data in link_info"))?;
    let bit_timing = info_data.bit_timing;
    let control_mode = info_data.control_mode.clone().unwrap_or_default();

    let fd_configuration = if control_mode.contains(&IF_CONTROL_MODE_CAN_FD.to_string()) {
        let data_bit_timing = info_data.can_fd_bit_timing.ok_or_else(|| anyhow::Error::msg("Missing data_bittiming for CAN FD interface"))?;
        CanFD::Enabled(data_bit_timing)
    } else {
        CanFD::Disabled
    };
    Ok(CanInterfaceConfiguration {
        bit_timing,
        fd: fd_configuration,
    })
}

#[cfg(test)]
mod tests {

    mod parsing {
        use opendut_model::util::net::NetworkInterfaceName;
        use crate::service::network_interface::manager::can::{convert_link_info_to_can_configuration, IpLinkShowOutputForCan};
        const IP_LINK_OUTPUT_CAN_FD_5MBIT: &str = r#"[{"ifindex":3,"ifname":"can0","flags":["NOARP","UP","LOWER_UP","ECHO"],"mtu":72,"qdisc":"pfifo_fast","operstate":"UP","linkmode":"DEFAULT","group":"default","txqlen":10,"link_type":"can","promiscuity":0,"allmulti":0,"min_mtu":0,"max_mtu":0,"linkinfo":{"info_kind":"can","info_data":{"ctrlmode":["FD"],"ctrlmode_supported":["LOOPBACK","LISTEN-ONLY","BERR-REPORTING","FD","FD-NON-ISO","CC-LEN8-DLC"],"state":"ERROR-ACTIVE","berr_counter":{"tx":0,"rx":0},"restart_ms":0,"bittiming":{"bitrate":1000000,"sample_point":"0.500","tq":25,"prop_seg":9,"phase_seg1":10,"phase_seg2":20,"sjw":10,"brp":1},"bittiming_const":{"name":"mcp251xfd","tseg1":{"min":2,"max":256},"tseg2":{"min":1,"max":128},"sjw":{"min":1,"max":128},"brp":{"min":1,"max":256},"brp_inc":1},"data_bittiming":{"bitrate":5000000,"sample_point":"0.500","tq":25,"prop_seg":1,"phase_seg1":2,"phase_seg2":4,"sjw":2,"brp":1},"data_bittiming_const":{"name":"mcp251xfd","tseg1":{"min":1,"max":32},"tseg2":{"min":1,"max":16},"sjw":{"min":1,"max":16},"brp":{"min":1,"max":256},"brp_inc":1},"clock":40000000}},"num_tx_queues":1,"num_rx_queues":1,"gso_max_size":65536,"gso_max_segs":65535,"tso_max_size":65536,"tso_max_segs":65535,"gro_max_size":65536,"parentbus":"spi","parentdev":"spi0.0"}]"#;
        const IP_LINK_OUTPUT_CAN_1MBIT: &str = r#"[{"ifindex":3,"ifname":"can0","flags":["NOARP","UP","LOWER_UP","ECHO"],"mtu":16,"qdisc":"pfifo_fast","operstate":"UP","linkmode":"DEFAULT","group":"default","txqlen":10,"link_type":"can","promiscuity":0,"allmulti":0,"min_mtu":0,"max_mtu":0,"linkinfo":{"info_kind":"can","info_data":{"ctrlmode_supported":["LOOPBACK","LISTEN-ONLY","BERR-REPORTING","FD","FD-NON-ISO","CC-LEN8-DLC"],"state":"ERROR-ACTIVE","berr_counter":{"tx":0,"rx":0},"restart_ms":0,"bittiming":{"bitrate":500000,"sample_point":"0.875","tq":25,"prop_seg":34,"phase_seg1":35,"phase_seg2":10,"sjw":5,"brp":1},"bittiming_const":{"name":"mcp251xfd","tseg1":{"min":2,"max":256},"tseg2":{"min":1,"max":128},"sjw":{"min":1,"max":128},"brp":{"min":1,"max":256},"brp_inc":1},"data_bittiming_const":{"name":"mcp251xfd","tseg1":{"min":1,"max":32},"tseg2":{"min":1,"max":16},"sjw":{"min":1,"max":16},"brp":{"min":1,"max":256},"brp_inc":1},"clock":40000000}},"num_tx_queues":1,"num_rx_queues":1,"gso_max_size":65536,"gso_max_segs":65535,"tso_max_size":65536,"tso_max_segs":65535,"gro_max_size":65536,"parentbus":"spi","parentdev":"spi0.0"}]"#;
        const IP_LINK_OUTPUT_VCAN: &str = r#"[{"ifindex":7,"ifname":"vcan0","flags":["NOARP","UP","LOWER_UP"],"mtu":72,"qdisc":"noqueue","operstate":"UNKNOWN","group":"default","txqlen":1000,"link_type":"can","promiscuity":0,"min_mtu":0,"max_mtu":0,"linkinfo":{"info_kind":"vcan"},"num_tx_queues":1,"num_rx_queues":1,"gso_max_size":65536,"gso_max_segs":65535,"addr_info":[]}]"#;
        #[test]
        fn parse_ip_link_output() {
            let parsed: Vec<IpLinkShowOutputForCan> = serde_json::from_str(IP_LINK_OUTPUT_CAN_FD_5MBIT).expect("Failed to parse JSON");
            assert_eq!(parsed.len(), 1);
            let parsed: Vec<IpLinkShowOutputForCan> = serde_json::from_str(IP_LINK_OUTPUT_CAN_1MBIT).expect("Failed to parse JSON");
            assert_eq!(parsed.len(), 1);
            let parsed: Vec<IpLinkShowOutputForCan> = serde_json::from_str(IP_LINK_OUTPUT_VCAN).expect("Failed to parse JSON");
            assert_eq!(parsed.len(), 1);
        }

        #[test]
        fn parse_ip_link_output_can_standard_settings() -> anyhow::Result<()> {
            let parsed: Vec<IpLinkShowOutputForCan> = serde_json::from_str(IP_LINK_OUTPUT_VCAN).expect("Failed to parse JSON");
            let ip_link = &parsed[0];
            let name = NetworkInterfaceName::try_from(ip_link._interface_name.clone()).expect("Failed to convert interface name");

            let can_configuration = convert_link_info_to_can_configuration(name.clone(), &ip_link.link_info);
            assert!(can_configuration.is_err(), "Expected error for non-CAN interface");
            Ok(())
        }

        #[test]
        fn parse_ip_link_output_can_fd_settings() {
            let parsed: Vec<IpLinkShowOutputForCan> = serde_json::from_str(IP_LINK_OUTPUT_CAN_FD_5MBIT).expect("Failed to parse JSON");
            let can_info = &parsed[0].link_info;
            assert_eq!(can_info.info_kind, "can");
            let info_data = can_info.info_data.as_ref().expect("Missing info_data");
            let bit_timing = &info_data.bit_timing;
            let control_mode = info_data.control_mode.clone().expect("Missing control_mode");
            assert_eq!(bit_timing.bitrate, 1000000);
            assert_eq!(bit_timing.sample_point, 0.5);
            assert_eq!(control_mode.len(), 1);
            let data_bit_timing = info_data.can_fd_bit_timing.as_ref().expect("Missing data_bittiming");
            assert_eq!(data_bit_timing.bitrate, 5000000);
            assert_eq!(data_bit_timing.sample_point, 0.5);
        }

    }
}