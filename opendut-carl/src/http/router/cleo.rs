use axum::body::StreamBody;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum_server_dual_protocol::tokio_util::io::ReaderStream;
use http::{header, StatusCode};
use crate::{CarlInstallDirectory};

use crate::util::{CLEO_IDENTIFIER, CleoArch};

pub async fn download_cleo(
    Path(architecture): Path<CleoArch>,
    State(carl_install_directory): State<CarlInstallDirectory>,
) -> impl IntoResponse {
    let cleo_file_name = format!("{}-{}.tar.gz", &architecture.distribution_name(), crate::app_info::CRATE_VERSION);
    let cleo_dir = carl_install_directory.path.join(CLEO_IDENTIFIER).join(&cleo_file_name);

    let file = match tokio::fs::File::open(cleo_dir).await {
        Ok(file) => { file }
        Err(_) => { return StatusCode::NOT_FOUND.into_response(); }
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let content_disposition = format!("attachment; filename=\"{}\"", cleo_file_name);
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

    use crate::util::{CLEO_IDENTIFIER, CleoArch};
    use crate::router::cleo::download_cleo;

    #[tokio::test()]
    async fn download_cleo_succeeds() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let dir = temp.child(CLEO_IDENTIFIER);
        fs::create_dir_all(&dir).expect("Unable to create dir.");

        let cleo_file_name = format!("{}-{}.tar.gz", &CleoArch::X86_64.distribution_name(), crate::app_info::CRATE_VERSION);

        let tar_file = dir.child(&cleo_file_name);
        File::create(tar_file.to_path_buf())?;
       
        let cleo_install_path = temp.to_path_buf();
        let cleo_state = State::<CarlInstallDirectory>(CarlInstallDirectory { path: cleo_install_path });
        let cleo = download_cleo(Path(CleoArch::X86_64), cleo_state).await;
        let response = cleo.into_response();
        let header = response.headers().get(header::CONTENT_DISPOSITION).unwrap();
        let expected_header = format!("attachment; filename=\"{}\"", cleo_file_name);
        assert_that!(header.clone().to_str()?, eq(expected_header.as_str()));
        
        Ok(())
    }
}