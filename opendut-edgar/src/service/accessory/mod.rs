pub mod manson_hcs3304;
pub mod accessory_manager;

use tokio::sync::watch;

pub trait Accessory {

    fn deploy(&mut self);
    fn undeploy(&mut self);
    fn get_termination_channel(&self) -> &watch::Receiver<bool>;

    fn termination_requested(&self) -> bool {
        self.get_termination_channel().has_changed().unwrap_or(true)
    }
}
