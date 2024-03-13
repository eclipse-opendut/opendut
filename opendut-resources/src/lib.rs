//! This crate provides the foundations for working with resources within the opendut universe
//! and is a complement to the opendut-types crate.
//!
//! # Features
//! Enable or disable features according to your needs and in order to optimize compile time and binary size.
//!
//! | Feature   | Default  | Description                                                                                  |
//! | ----------|:--------:| ---------------------------------------------------------------------------------------------|
//! | broker    | &#x2717; | Enables the [`broker`] module which provides infrastructure for central resource management. |
//! | replica   | &#x2717; | Enables the [`replica`] module to consume events for distributed resource replication.       |
//! | derive    | &#x2714; | Enables the derive macro to implement traits for resources.                                  |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!

pub mod prelude {
    pub use uuid::Uuid;
    #[cfg(feature = "derive")]
    pub use opendut_resources_derive::ResourceRef;
    #[cfg(feature = "derive")]
    pub use opendut_resources_derive::Marshaller;
    pub use crate::resource;
    pub use crate::resource::{Resource, ResourceRef};
    pub use crate::resource::marshalling::Marshaller;
    pub use crate::resource::versioning::{RevisionHash, ROOT_REVISION_HASH};
}

pub mod resource;

#[cfg(feature = "broker")]
pub mod broker;

#[cfg(feature = "replica")]
pub mod replica;

#[cfg(feature = "repository")]
pub mod repository;

#[cfg(test)]
mod testkit;
