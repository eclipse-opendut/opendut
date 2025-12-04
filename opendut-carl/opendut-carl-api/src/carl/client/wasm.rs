use tonic::codegen::InterceptedService;

    use opendut_auth::public::{AuthInterceptor, Authentication};

    use crate::carl::cluster::ClusterManager;
    use crate::carl::InitializationError;
    use crate::carl::metadata::MetadataProvider;
    use crate::carl::peer::PeersRegistrar;
    #[cfg(feature="viper")]
    use crate::carl::viper::TestManager;

#[derive(Debug, Clone)]
pub struct CarlClient {
    pub cluster: ClusterManager<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
    pub metadata: MetadataProvider<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
    pub peers: PeersRegistrar<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
    #[cfg(feature="viper")]
    pub viper: TestManager<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
}

impl CarlClient {
    pub async fn create(url: url::Url, auth: Authentication) -> Result<CarlClient, InitializationError> {
        let scheme = url.scheme();
        if scheme != "https" {
            return Err(InitializationError::ExpectedHttpsScheme { given_scheme: scheme.to_owned() });
        }

        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(443_u16);

        let client = tonic_web_wasm_client::Client::new(format!("{scheme}://{host}:{port}"));
        let auth_interceptor = AuthInterceptor::new(auth);

        Ok(CarlClient {
            cluster: ClusterManager::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
            metadata: MetadataProvider::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
            peers: PeersRegistrar::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
            #[cfg(feature="viper")]
            viper: TestManager::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
        })
    }
}
