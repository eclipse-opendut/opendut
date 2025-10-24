use std::fmt::Display;
pub use cluster_manager::ClusterManagerFacade;
pub use metadata_provider::MetadataProviderFacade;
pub use peer_manager::PeerManagerFacade;
pub use peer_messaging_broker::PeerMessagingBrokerFacade;
pub use observer_messaging_broker::ObserverMessagingBrokerFacade;
pub use test_manager::TestManagerFacade;

mod cluster_manager;
mod metadata_provider;
mod peer_manager;
mod peer_messaging_broker;
mod observer_messaging_broker;
mod test_manager;
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


mod web {
    use std::task::{Context, Poll};
    use tonic::body::Body;
    use tonic::server::NamedService;
    use tonic_web::GrpcWebService;
    use tower::{Layer, Service};
    use tower_http::cors::Cors;

    pub fn enable<S>(service: S) -> CorsGrpcWeb<S>
    where
        S: Service<http::Request<Body>, Response = http::Response<Body>>,
    {
        let service = tower::layer::util::Stack::new(
                tonic_web::GrpcWebLayer::new(),
                tower_http::cors::CorsLayer::new(),
            )
            .layer(service);

        CorsGrpcWeb(service)
    }

    #[derive(Debug, Clone)]
    pub struct CorsGrpcWeb<S>(Cors<GrpcWebService<S>>);

    impl<S> Service<http::Request<Body>> for CorsGrpcWeb<S>
    where
        S: Service<http::Request<Body>, Response = http::Response<Body>>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = <Cors<GrpcWebService<S>> as Service<http::Request<Body>>>::Future;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            <Cors<GrpcWebService<S>> as Service<http::Request<Body>>>::poll_ready(&mut self.0, cx)
        }
        fn call(&mut self, req: http::Request<Body>) -> Self::Future {
            self.0.call(req)
        }
    }

    impl<S: NamedService> NamedService for CorsGrpcWeb<S> {
        const NAME: &'static str = S::NAME;
    }
}
