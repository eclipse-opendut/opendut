use leptos::{ReadSignal, use_context};

pub use licenses::ComponentLicenses;
pub use licenses::get_licenses;
use opendut_carl_api::carl::wasm::CarlClient;

pub use crate::util::url::{UrlDecodable, UrlEncodable};

#[derive(thiserror::Error, Debug, Clone)]
pub enum ApiError {

    #[error("{message}")]
    HttpError {
        message: String,
    },

    #[error("{message}")]
    JsonParseError {
        message: String,
    },
}

pub fn use_carl() -> ReadSignal<CarlClient> {
    use_context::<ReadSignal<CarlClient>>()
        .expect("A client CARL should be provided in the context.")
}

mod licenses {
    use std::collections::BTreeMap;

    use gloo_net::http;
    use serde::Deserialize;

    use crate::api::ApiError;

    #[derive(Clone)]
    pub struct ComponentLicenses {
        pub name: String,
        pub licenses: Vec<ComponentDependencyLicense>,
    }

    #[derive(Clone)]
    pub struct ComponentDependencyLicense {
        pub name: String,
        pub version: String,
        pub licenses: Vec<String>,
    }

    #[derive(Deserialize)]
    pub struct LicensesIndexJson {
        pub carl: String,
        pub edgar: String,
        pub lea: String,
    }

    #[derive(Deserialize)]
    pub struct LicenseJsonEntry {
        pub licenses: Vec<String>,
    }

    pub async fn get_licenses() -> Result<Vec<ComponentLicenses>, ApiError> {

        log::debug!("Requesting licenses.");

        let index = {
            let licenses_index = http::Request::get("/api/licenses")
                .send().await
                .map_err(|cause| ApiError::HttpError {
                    message: format!("Failed to request the licenses index file due to: {}", cause),
                })?
                .json::<LicensesIndexJson>().await
                .map_err(|cause| ApiError::JsonParseError {
                    message: format!("Failed to parse the licenses index file due to: {}", cause),
                })?;

            [
                ("carl", format!("/api/licenses/{}", licenses_index.carl)),
                ("edgar", format!("/api/licenses/{}", licenses_index.edgar)),
                ("lea", format!("/api/licenses/{}", licenses_index.lea))
            ]
        };

        let mut result = Vec::<ComponentLicenses>::new();
        for (name, path) in index {
            let licenses = http::Request::get(&path)
                .send().await
                .map_err(|cause| ApiError::HttpError {
                    message: format!("Failed to request the licenses file due to: {}", cause),
                })?
                .json::<BTreeMap<String, LicenseJsonEntry>>().await
                .map_err(|cause| ApiError::HttpError {
                    message: format!("Failed to parse the licenses file due to: {}", cause),
                })?;

            let dependencies = licenses.iter()
                .map(|(dependency, licenses) | {
                    let split = dependency.split(" ").collect::<Vec<_>>();
                    let dependency_name = split[0];
                    let dependency_version = split[1];
                    ComponentDependencyLicense {
                        name: String::from(dependency_name),
                        version: String::from(dependency_version),
                        licenses: licenses.licenses.clone(),
                    }
                }).collect::<Vec<ComponentDependencyLicense>>();

            result.push(ComponentLicenses {
                name: String::from(name),
                licenses: dependencies,
            });
        }

        log::debug!("Retrieved licenses.");

        Ok(result)
    }
}
