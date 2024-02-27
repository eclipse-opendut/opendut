use std::ops::Not;
use std::time::Duration;

use anyhow::bail;
use config::Config;
use opendut_carl_api::carl::CarlClient;

use opendut_util::project;

pub async fn connect(settings: &Config) -> anyhow::Result<CarlClient> {
    let host = settings.get_string("network.carl.host")?;
    let port = u16::try_from(settings.get_int("network.carl.port")?)?;
    let ca_cert_path = project::make_path_absolute(settings.get_string("network.tls.ca.certificate")?)?;
    let domain_name_override = settings.get_string("network.tls.domain.name.override")?;
    let domain_name_override = domain_name_override.is_empty().not().then_some(domain_name_override);

    let mut carl = CarlClient::create(&host, port, ca_cert_path, domain_name_override, settings)?;

    let retries = settings.get_int("network.connect.retries")?;
    let interval = Duration::from_millis(u64::try_from(settings.get_int("network.connect.interval.ms")?)?);

    for retries_left in (0..retries).rev() {
        match carl.metadata.version().await {
            Ok(version) => {
                log::info!("Connected to CARL with version {}.", version.name);
                return Ok(carl);
            }
            Err(cause) => {
                if retries_left > 0 {
                    log::warn!("Could not connect to CARL at '{host}:{port}'. Retrying in {interval} ms. {retries_left} retries left.\n  {cause}", interval=interval.as_millis());
                    tokio::time::sleep(interval).await;
                }
            }
        }
    }
    bail!("Failed to connect to CARL after {retries}*{interval} ms.", interval=interval.as_millis());
}
