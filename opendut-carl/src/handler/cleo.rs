use axum::body::StreamBody;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum_server_dual_protocol::tokio_util::io::ReaderStream;
use http::{header, StatusCode};
use opendut_util::project;
use crate::{CarlInstallDirectory};

use crate::util::{CLEO_TARGET_DIRECTORY, CleoArch};

pub async fn download_cleo(
    Path(architecture): Path<CleoArch>,
    State(carl_install_directory): State<CarlInstallDirectory>,
) -> impl IntoResponse {
    let mut file_name = architecture.file_name();
    let mut content_type = "application/gzip";

    let cleo_dir = if project::is_running_in_development() {
        if file_name !=  CleoArch::Development.file_name() {
            return StatusCode::NOT_FOUND.into_response();
        }
        content_type = "application/octet-stream";
        carl_install_directory.path.join("target/debug/opendut-cleo")
    } else {
        file_name = format!("{}.tar.gz", &file_name);
        carl_install_directory.path.join(CLEO_TARGET_DIRECTORY).join(&file_name)
    };
    println!("Cleo install directory: {:?}", cleo_dir);

    let file = match tokio::fs::File::open(cleo_dir).await {
        Ok(file) => { file }
        Err(_) => { return StatusCode::NOT_FOUND.into_response(); }
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let content_disposition = format!("attachment; filename=\"{}\"", file_name);
    let headers = [
        (header::CONTENT_TYPE, content_type),
        (
            header::CONTENT_DISPOSITION,
            content_disposition.as_str(),
        ),
    ];
    (headers, body).into_response()
}

#[cfg(test)]
mod test {
    use assert_fs::fixture::{FileTouch, PathChild};
    use assert_fs::TempDir;
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use googletest::assert_that;
    use googletest::matchers::eq;
    use http::header;
    use crate::CarlInstallDirectory;

    use crate::util::{CleoArch};
    use crate::handler::cleo::download_cleo;

    #[tokio::test()]
    async fn download_cleo_development_succeeds() -> anyhow::Result<()> {
        let temp = TempDir::new().unwrap();

        let dir = temp.child("target/debug/opendut-cleo");
        dir.touch().unwrap();
       
        let cleo_install_path = temp.to_path_buf();
        let cleo_state = State::<CarlInstallDirectory>(CarlInstallDirectory { path: cleo_install_path });
        let cleo = download_cleo(Path(CleoArch::Development), cleo_state).await;
        let response = cleo.into_response();
        let header = response.headers().get(header::CONTENT_DISPOSITION).unwrap();
        let expected_header = format!("attachment; filename=\"{}\"", CleoArch::Development.file_name());
        assert_that!(header.clone().to_str().unwrap(), eq(expected_header.as_str()));
        
        Ok(())
    }
}