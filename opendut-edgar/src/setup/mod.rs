use std::net::Ipv4Addr;

pub use crate::setup::runner::RunMode;

pub mod constants;
mod runner;
pub mod start;
mod task;
#[allow(non_camel_case_types)]
mod tasks;
mod util;

#[derive(Clone, Debug)]
pub enum Router { Local, Remote(Ipv4Addr) }
