use oauth2::{HttpRequest, HttpResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<T>
    where
        T: std::error::Error + 'static,
{
    #[error("AuthReqwest Error: Request failed")]
    AuthReqwest(#[source] T),
}

pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, Error<reqwest::Error>> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(Error::AuthReqwest)?;
    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::AuthReqwest)?;
    let response = client.execute(request).await.map_err(Error::AuthReqwest)?;
    let status_code = response.status();
    let headers = response.headers().to_owned();
    let data = response.bytes().await.map_err(Error::AuthReqwest)?;
    Ok(HttpResponse {
        status_code,
        headers,
        body: data.to_vec(),
    })
}
