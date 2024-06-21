use std::sync::Arc;
use std::time::Duration;
use opentelemetry::{global, KeyValue};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, trace};
use opendut_types::cluster::PeerClusterAssignment;

pub async fn cluster_ping(peers: Vec<PeerClusterAssignment>, ping_interval_ms: Duration) {
    let meter = global::meter(opendut_util::logging::DEFAULT_METER_NAME);
    let rtt = meter.f64_gauge("round_trip_time").init();

    let rtt_mutex = Arc::new(Mutex::new(rtt));

    tokio::spawn(async move {
        let data = [1, 2, 3, 4];
        let options = ping_rs::PingOptions { ttl: 128, dont_fragment: true };
        loop {
            sleep(ping_interval_ms).await;
            let timeout = Duration::from_secs(1); //TODO make configurable
            for peer in peers.clone() {
                let remote_address = peer.vpn_address;
                let result = ping_rs::send_ping(&remote_address, timeout, &data, Some(&options));
                match result {
                    Ok(reply) => {
                        rtt_mutex.lock().await
                            .record(reply.rtt as f64, &[KeyValue::new("peer_ip_address", remote_address.to_string())]);
                        trace!("Reply from {}: bytes={} time={}ms TTL={}", reply.address, data.len(), reply.rtt, options.ttl)
                    },
                    Err(cause) => error!("Error while pinging peer {peer_id} with IP {peer_ip}: {cause:?}", peer_id=peer.peer_id, peer_ip=remote_address)
                }
            }
        }
    });
}