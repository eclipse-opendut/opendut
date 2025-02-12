use std::net::IpAddr;
use std::ops::Not;
use std::{collections::HashMap, sync::Arc};
use std::time::Duration;
use opendut_types::peer::PeerId;
use opentelemetry::{global, KeyValue};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, trace};


pub async fn spawn_cluster_ping(peers: HashMap<PeerId, IpAddr>, ping_interval_ms: Duration) {

    let meter = global::meter(opendut_util::telemetry::DEFAULT_METER_NAME);
    let rtt = meter.f64_gauge("round_trip_time").build();

    let rtt_mutex = Arc::new(Mutex::new(rtt));

    let data = [1, 2, 3, 4];
    let options = ping_rs::PingOptions { ttl: 128, dont_fragment: true };

    let mut last_ping_was_successful = false;

    loop {
        sleep(ping_interval_ms).await;
        let timeout = Duration::from_secs(1); //TODO make configurable

        for (peer_id, vpn_address) in &peers {
            let remote_address = vpn_address;
            let result = ping_rs::send_ping(remote_address, timeout, &data, Some(&options));

            match result {
                Ok(reply) => {
                    rtt_mutex.lock().await
                        .record(reply.rtt as f64, &[KeyValue::new("peer_ip_address", remote_address.to_string())]);

                    if last_ping_was_successful.not() {
                        trace!("Reply from {}: bytes={} time={}ms TTL={}", reply.address, data.len(), reply.rtt, options.ttl);
                        last_ping_was_successful = true;
                    }
                },
                Err(cause) => {
                    error!("Error while pinging peer {peer_id} with IP {peer_ip}: {cause:?}", peer_ip=remote_address);
                    last_ping_was_successful = false;
                }
            }
        }
    }
}
