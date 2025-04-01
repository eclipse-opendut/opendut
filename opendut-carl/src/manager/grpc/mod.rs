use std::fmt::Display;

pub use cluster_manager::ClusterManagerFacade;
pub use metadata_provider::MetadataProviderFacade;
pub use peer_manager::PeerManagerFacade;
pub use peer_messaging_broker::PeerMessagingBrokerFacade;

mod cluster_manager;
mod metadata_provider;
mod peer_manager;
mod peer_messaging_broker;
mod error;

pub trait ExtractOrInvalidArgument<A, B>
where
    B: TryFrom<A>,
    B::Error: Display
{
    fn extract_or_invalid_argument(self, field: impl Into<String> + Clone) -> Result<B, tonic::Status>;
}

impl <A, B> ExtractOrInvalidArgument<A, B> for Option<A>
where
    B: TryFrom<A>,
    B::Error: Display
{
    fn extract_or_invalid_argument(self, field: impl Into<String> + Clone) -> Result<B, tonic::Status> {
        self
            .ok_or_else(|| tonic::Status::invalid_argument(format!("Field '{}' not set", Clone::clone(&field).into())))
            .and_then(|value| {
                B::try_from(value).map_err(|cause| {
                    tonic::Status::invalid_argument(format!("Field '{}' is not valid: {}", field.into(), cause))
                })
            })
    }
}

macro_rules! extract {
    ($spec:expr) => {
        crate::manager::grpc::ExtractOrInvalidArgument::extract_or_invalid_argument($spec, stringify!($spec))
    };
}

pub(crate) use extract;
