use crate::service::accessory::Accessory;
use tokio::sync::watch::Receiver;
use tracing::info;

pub struct MansonHcs3304 {
    rx_termination_channel: Receiver<bool>,
    serial_port: String,
    mqtt_broker_url: Option<url::Url>
}

impl MansonHcs3304 {
    pub fn new(rx_termination_channel: Receiver<bool>, serial_port: String, mqtt_broker_url: Option<url::Url>) -> Self {
        Self { 
            rx_termination_channel,
            serial_port,
            mqtt_broker_url,
        }
    }
}

impl Accessory for MansonHcs3304 {
    fn deploy(&mut self) {
        info!("Deployment of Manson HCS-3304 triggered");
    }

    fn undeploy(&mut self) {
        info!("Undeployment of Manson HCS-3304 triggered");
    }

    fn get_termination_channel(&self) -> &tokio::sync::watch::Receiver<bool> {
        &self.rx_termination_channel
    }
}