use std::collections::HashSet;
use leptos::prelude::*;
use tracing::warn;
use opendut_lea_components::{CollapsableInfo, Ior, MultipleSelectionTable, MultipleSelectionTableCell, MultipleSelectionTableRow, TableDisplayType};
use opendut_model::peer::PeerDescriptor;
use opendut_model::topology::{DeviceDescriptor, DeviceId};
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::components::UserNetworkInterfaceConfiguration;

pub type DeviceSelectionError = String;
pub type DeviceSelection = Ior<DeviceSelectionError, HashSet<DeviceId>>;

#[component]
pub fn DeviceSelector(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
    is_disabled: Signal<bool>,
) -> impl IntoView {

    let (getter, setter) = create_slice(
        cluster_descriptor,
        |config| Clone::clone(&config.devices),
        |config, input| {
            config.devices = input;
        },
    );

    let devices = Signal::derive(move || {
        let mut rows = peers.get().into_iter()
            .flat_map(|peer| {
                let PeerDescriptor {
                    id: peer_id,
                    name: peer_name,
                    location: peer_location,
                    network,
                    topology,
                    ..
                } = peer;

                topology.devices.into_iter().map(move |device|{
                    let DeviceDescriptor {
                        id: device_id,
                        name: device_name,
                        description: device_description,
                        interface: device_interface,
                        tags
                    } = device;

                    let network_interface = network.interfaces.iter()
                        .find(|interface| interface.id == device_interface)
                        .cloned();

                    let network_interface_text = match network_interface {
                        Some(network_interface) => {
                            let interface_name = network_interface.name.name();
                            let configuration_display_name = UserNetworkInterfaceConfiguration::from(network_interface.configuration).display_name();
                            format!("{} ({})", interface_name, configuration_display_name)

                        }
                        None => {
                            warn!("The network interface <{}> associated with device {} <{}> does not have a NetworkInterfaceDescriptor on peer <{}>.", device_interface, device_name, device_id, peer_id);
                            format!("Unknown interface ({})", device.interface)
                        }
                    };
                    
                    let device_details = vec![
                        CollapsableInfo { label: String::from("ID"), value: device_id.to_string() },
                        CollapsableInfo { label: String::from("Peer ID"), value: peer_id.to_string() },
                        CollapsableInfo { label: String::from("Interface"), value: network_interface_text },
                        CollapsableInfo { label: String::from("Description"), value: device_description.unwrap_or_default().to_string() },
                    ];

                    let device_name = vec![device_name.to_string()];
                    let peer_name = vec![peer_name.to_string()];
                    let peer_location = vec![Clone::clone(&peer_location).unwrap_or_default().value()];
                    let tags = tags.iter().map(|tag| String::from(tag.value())).collect::<Vec<_>>();

                    MultipleSelectionTableRow {
                        id: device_id,
                        cells: vec![
                            MultipleSelectionTableCell { value: device_name, display_type: TableDisplayType::Text },
                            MultipleSelectionTableCell { value: peer_name, display_type: TableDisplayType::Text },
                            MultipleSelectionTableCell { value: peer_location, display_type: TableDisplayType::Text },
                            MultipleSelectionTableCell { value: tags, display_type: TableDisplayType::Tag },
                        ],
                        details: device_details,
                    }
                })
            }).collect::<Vec<_>>();

        rows.sort_by(|a, b| {
            a.cells[0].value[0].to_lowercase()
                .cmp(&b.cells[0].value[0].to_lowercase())
        });
        rows
    });

    let header = vec![
        String::from("Name"),
        String::from("Peer"),
        String::from("Peer Location"),
        String::from("Tags"),
    ];

    view! {
        <MultipleSelectionTable
            header
            rows=devices
            getter
            setter
            is_disabled
        />
    }
}
