//! This crate serves as a library for opendut-lea.
//!
//! All components or types which are generic enough
//! so that they could be used in another web-UI should
//! be extracted into this library.
//!
//! The goal is a clearer seperation to help with
//! structuring the code, but in particular also to
//! reduce compile times. By moving code into a separate
//! crate, we benefit from incremental compilation.

pub mod components;
mod util;

pub use util::*;
