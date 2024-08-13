use std::net::Ipv4Addr;

pub use crate::setup::runner::RunMode;

mod constants;
mod runner;
pub mod start;
mod task;
#[cfg(not(target_arch = "arm"))]
mod setup_plugin;
#[cfg(not(target_arch = "arm"))]
mod plugin_runtime;
#[allow(non_camel_case_types)]
mod tasks;
mod util;

#[derive(Clone, Debug)]
pub enum Leader { Local, Remote(Ipv4Addr) }

#[derive(Clone, Debug)]
struct User { pub name: String }
impl User {
    fn is_root(&self) -> bool {
        self.name == "root"
    }
}
