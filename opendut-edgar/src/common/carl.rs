use std::net::IpAddr;

use tracing::{debug, info};

use opendut_carl_api::carl::broker::stream_header;
use opendut_carl_api::carl::{broker, CarlClient};
use opendut_model::peer::PeerId;


pub async fn open_stream(
    self_id: PeerId,
    remote_address: &IpAddr,
    carl: &mut CarlClient,
) -> anyhow::Result<(broker::Downstream, broker::Upstream), broker::error::OpenStream> {
    debug!("Opening peer messaging stream...");

    let extra_headers = stream_header::ExtraHeaders {
        client_version: Some(stream_header::PeerVersion {
            value: crate::app_info::PKG_VERSION.to_owned()
        }),
    };
    let (rx_inbound, tx_outbound) = carl.broker.open_stream(self_id, remote_address, extra_headers).await?;

    tx_outbound.send(broker::UpstreamMessage {
        context: None,
        payload: broker::UpstreamMessagePayload::Ping,
    }).await
        .map_err(|source| broker::error::OpenStream { message: format!("Error while sending initial ping: {source}") })?;

    info!("Peer messaging stream opened.");
    Ok((rx_inbound, tx_outbound))
}

pub async fn log_version_compatibility(carl: &mut CarlClient) -> anyhow::Result<()> {
    use opendut_carl_api::carl::metadata::version_compatibility::*;

    log_version_compatibility_with_carl(
        VersionCompatibilityInfo {
            own_version: crate::app_info::PKG_VERSION,
            own_name: String::from("EDGAR"),
            upgrade_hint: Some(String::from(
                "You can update to the newest version of EDGAR by following the steps here: https://opendut.eclipse.dev/book/user-manual/edgar/setup.html"
            )),
        },
        carl
    ).await?;

    Ok(())
}
