use axum::body::StreamBody;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum_server_dual_protocol::tokio_util::io::ReaderStream;
use http::{header, StatusCode};
use crate::http::state::CarlInstallDirectory;
use crate::util::{EDGAR_IDENTIFIER, EdgarArch};

pub async fn download_edgar(
    Path(architecture): Path<EdgarArch>,
    State(carl_install_directory): State<CarlInstallDirectory>,
) -> impl IntoResponse {

    let file_name = format!("{}-{}.tar.gz", &architecture.distribution_name(), crate::app_info::CRATE_VERSION);
    let edgar_dir = carl_install_directory.path.join(EDGAR_IDENTIFIER).join(&file_name);

    let file = match tokio::fs::File::open(edgar_dir).await {
        Ok(file) => { file }
        Err(_) => { return StatusCode::NOT_FOUND.into_response(); }
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let content_disposition = format!("attachment; filename=\"{}\"", file_name);
    let headers = [
        (header::CONTENT_TYPE, "application/gzip"),
        (
            header::CONTENT_DISPOSITION,
            content_disposition.as_str(),
        ),
    ];
    (headers, body).into_response()
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

        let edgar_file_name = format!("{}-{}.tar.gz", &EdgarArch::X86_64.distribution_name(), crate::app_info::CRATE_VERSION);

        let tar_file = dir.child(&edgar_file_name);
        File::create(tar_file.to_path_buf())?;

        let cleo_install_path = temp.to_path_buf();
        let cleo_state = State::<CarlInstallDirectory>(CarlInstallDirectory { path: cleo_install_path });
        let cleo = download_edgar(Path(EdgarArch::X86_64), cleo_state).await;
        let response = cleo.into_response();
        let header = response.headers().get(header::CONTENT_DISPOSITION).unwrap();
        let expected_header = format!("attachment; filename=\"{}\"", edgar_file_name);
        assert_that!(header.clone().to_str()?, eq(expected_header.as_str()));

        Ok(())
    }
}