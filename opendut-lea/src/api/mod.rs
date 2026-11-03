pub use licenses::get_licenses;
pub use licenses::ComponentLicenses;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ApiError {
    #[error("{message}")]
    HttpError { message: String },

    #[error("{message}")]
    JsonParseError { message: String },
}

mod licenses {
    use gloo_net::http;
    use serde::Deserialize;
    use serde_spdx::spdx::v_2_3::Spdx;
    use tracing::debug;

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
        pub cleo: String,
        pub edgar: String,
        pub lea: String,
    }

    pub async fn get_licenses() -> Result<Vec<ComponentLicenses>, ApiError> {
        debug!("Requesting licenses.");

        let index = {
            let licenses_index = http::Request::get("/api/licenses")
                .send()
                .await
                .map_err(|cause| ApiError::HttpError {
                    message: format!(
                        "Failed to request the licenses index file due to: {cause}"
                    ),
                })?
                .json::<LicensesIndexJson>()
                .await
                .map_err(|cause| ApiError::JsonParseError {
                    message: format!("Failed to parse the licenses index file due to: {cause}"),
                })?;

            [
                ("carl", format!("/api/licenses/{}", licenses_index.carl)),
                ("edgar", format!("/api/licenses/{}", licenses_index.edgar)),
                ("cleo", format!("/api/licenses/{}", licenses_index.cleo)),
                ("lea", format!("/api/licenses/{}", licenses_index.lea)),
            ]
        };

        let mut result = Vec::<ComponentLicenses>::new();
        for (name, path) in index {
            let licenses = http::Request::get(&path)
                .send()
                .await
                .map_err(|cause| ApiError::HttpError {
                    message: format!("Failed to request the licenses file due to: {cause}"),
                })?
                .json::<Spdx>()
                .await
                .map_err(|cause| ApiError::HttpError {
                    message: format!("Failed to parse the licenses file due to: {cause}"),
                })?;

            let mut dependencies = licenses.packages
                .unwrap_or_default()
                .into_iter()
                .map(|dependency| {
                    ComponentDependencyLicense {
                        name: dependency.name,
                        version: dependency.version_info.unwrap_or_else(|| String::from("No version found.")),
                        licenses: vec![dependency.license_declared
                            .unwrap_or_else(|| String::from("No license found."))],
                    }
                })
                .collect::<Vec<ComponentDependencyLicense>>();
            dependencies.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

            result.push(ComponentLicenses {
                name: String::from(name),
                licenses: dependencies,
            });
        }

        debug!("Retrieved licenses.");

        Ok(result)
    }
}
