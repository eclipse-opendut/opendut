use axum::body::{Body};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::{header, HeaderValue, Request};
use tower_http::services::ServeFile;
use crate::http::state::CarlInstallDirectory;
use crate::util::{EDGAR_IDENTIFIER, EdgarArch};

pub async fn download_edgar(
    Path(architecture): Path<EdgarArch>,
    State(carl_install_directory): State<CarlInstallDirectory>,
) -> impl IntoResponse {

    let file_name = format!("{}-{}.tar.gz", &architecture.distribution_name(), crate::app_info::CRATE_VERSION);
    let edgar_path = carl_install_directory.path.join(EDGAR_IDENTIFIER).join(&file_name);

    let mut response = ServeFile::new_with_mime(edgar_path, &mime::APPLICATION_OCTET_STREAM)
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
    use std::fs;
    use std::fs::File;
    use assert_fs::fixture::PathChild;
    use assert_fs::TempDir;
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use googletest::assert_that;
    use googletest::matchers::eq;
    use http::header;
    use crate::CarlInstallDirectory;
    use crate::http::router::edgar::download_edgar;

    use crate::util::{EDGAR_IDENTIFIER, EdgarArch};

    #[tokio::test()]
    async fn download_edgar_succeeds() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let dir = temp.child(EDGAR_IDENTIFIER);
        fs::create_dir_all(&dir).expect("Unable to create dir.");

        let file_name = format!("{}-{}.tar.gz", &EdgarArch::X86_64.distribution_name(), crate::app_info::CRATE_VERSION);

        let tar_file = dir.child(&file_name);
        File::create(&tar_file)?;

        let state = State::<CarlInstallDirectory>(CarlInstallDirectory { path: temp.to_path_buf() });

        let result = download_edgar(Path(EdgarArch::X86_64), state).await;

        let result = result.into_response();
        let header = result.headers()
            .get(header::CONTENT_DISPOSITION)
            .unwrap();

        let expected_header = format!("attachment; filename=\"{}\"", file_name);
        assert_that!(header.to_str()?, eq(&expected_header));

        Ok(())
    }
}
