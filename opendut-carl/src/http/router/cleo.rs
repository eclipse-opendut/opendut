use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::{header, HeaderValue, Request};
use tower_http::services::ServeFile;

use crate::util::{CleoArch, CLEO_IDENTIFIER};
use crate::CarlInstallDirectory;

pub async fn download_cleo(
    Path(architecture): Path<CleoArch>,
    State(carl_install_directory): State<CarlInstallDirectory>,
) -> impl IntoResponse {
    let file_name = format!("{}-{}.tar.gz", &architecture.distribution_name(), crate::app_info::PKG_VERSION);
    let file_path = carl_install_directory.path.join(CLEO_IDENTIFIER).join(&file_name);

    let mut response = ServeFile::new_with_mime(file_path, &mime::APPLICATION_OCTET_STREAM)
        .try_call(Request::new(Body::empty())).await
        .unwrap();

    response.headers_mut()
        .append(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_name)).unwrap()
        );

    response
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;
    use std::fs::File;

    use assert_fs::fixture::PathChild;
    use assert_fs::TempDir;
    use axum::extract::{Path, State};
    use googletest::assert_that;
    use googletest::matchers::eq;
    use http::header;

    use crate::util::{CleoArch, CLEO_IDENTIFIER};
    use crate::CarlInstallDirectory;

    #[tokio::test()]
    async fn download_cleo_succeeds() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let dir = temp.child(CLEO_IDENTIFIER);
        fs::create_dir_all(&dir).expect("Unable to create dir.");

        let file_name = format!("{}-{}.tar.gz", &CleoArch::X86_64.distribution_name(), crate::app_info::PKG_VERSION);

        let tar_file = dir.child(&file_name);
        File::create(&tar_file)?;
       
        let install_path = temp.to_path_buf();
        let state = State::<CarlInstallDirectory>(CarlInstallDirectory { path: install_path });

        let result = download_cleo(Path(CleoArch::X86_64), state).await;

        let result = result.into_response();
        let header = result.headers()
            .get(header::CONTENT_DISPOSITION)
            .unwrap();

        let expected_header = format!("attachment; filename=\"{}\"", file_name);
        assert_that!(header.to_str()?, eq(&expected_header));

        Ok(())
    }
}
