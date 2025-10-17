use std::net::Ipv4Addr;

pub use crate::common::task::runner::RunMode;

pub mod cli;
mod constants;

pub mod start;

mod plugin;

#[allow(non_camel_case_types)]
mod tasks;

mod util;
pub mod write_configuration;

pub use util::user_confirmation_prompt;

#[derive(Clone, Debug)]
pub enum Leader { Local, Remote(Ipv4Addr) }

#[derive(Clone, Debug)]
struct User { pub name: String }
impl User {
    fn is_root(&self) -> bool {
        self.name == "root"
    }
}
