//! Code adapted from https://github.com/tokio-rs/axum/issues/2736#issuecomment-2256154646

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::ready;
use http::{header::CONTENT_TYPE, Request, Response};
use http_body::Body;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pin_project! {
    #[project = GrpcHttpMultiplexFutureEnumProjection]
    enum GrpcHttpMultiplexFutureEnum<FutureGrpc, FutureHttp> {
        Grpc {
            #[pin]
            future: FutureGrpc,
        },
        Http {
            #[pin]
            future: FutureHttp,
        },
    }
}

pin_project! {
    pub struct GrpcHttpMultiplexFuture<FutureGrpc, FutureHttp> {
        #[pin]
        future: GrpcHttpMultiplexFutureEnum<FutureGrpc, FutureHttp>,
    }
}

impl<ResponseBody, FutureGrpc, FutureHttp, ErrorGrpc, ErrorHttp> Future for GrpcHttpMultiplexFuture<FutureGrpc, FutureHttp>
where
    ResponseBody: Body,
    FutureGrpc: Future<Output = Result<Response<ResponseBody>, ErrorGrpc>>,
    FutureHttp: Future<Output = Result<Response<ResponseBody>, ErrorHttp>>,
    ErrorGrpc: Into<BoxError> + Send,
    ErrorHttp: Into<BoxError> + Send,
{
    type Output = Result<Response<ResponseBody>, Box<dyn std::error::Error + Send + Sync + 'static>>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.project() {
            GrpcHttpMultiplexFutureEnumProjection::Grpc { future } => {
                future.poll(context).map_err(Into::into)
            }
            GrpcHttpMultiplexFutureEnumProjection::Http { future } => {
                future.poll(context).map_err(Into::into)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrpcHttpMultiplexService<Grpc, Http> {
    grpc: Grpc,
    http: Http,
    grpc_ready: bool,
    http_ready: bool,
}

impl<ReqBody, ResBody, Grpc, Http> Service<Request<ReqBody>> for GrpcHttpMultiplexService<Grpc, Http>
where
    ResBody: Body,
    Grpc: Service<Request<ReqBody>, Response = Response<ResBody>>,
    Http: Service<Request<ReqBody>, Response = Response<ResBody>>,
    Grpc::Error: Into<BoxError> + Send,
    Http::Error: Into<BoxError> + Send,
{
    type Response = Grpc::Response;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = GrpcHttpMultiplexFuture<Grpc::Future, Http::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            match (self.grpc_ready, self.http_ready) {
                (true, true) => {
                    return Ok(()).into();
                }
                (false, _) => {
                    ready!(self.grpc.poll_ready(cx)).map_err(Into::into)?;
                    self.grpc_ready = true;
                }
                (_, false) => {
                    ready!(self.http.poll_ready(cx)).map_err(Into::into)?;
                    self.http_ready = true;
                }
            }
        }
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        assert!(self.grpc_ready);
        assert!(self.http_ready);

        if is_grpc_request(&request) {
            GrpcHttpMultiplexFuture {
                future: GrpcHttpMultiplexFutureEnum::Grpc {
                    future: self.grpc.call(request),
                },
            }
        } else {
            GrpcHttpMultiplexFuture {
                future: GrpcHttpMultiplexFutureEnum::Http {
                    future: self.http.call(request),
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrpcHttpMultiplexLayer<Http> {
    http: Http,
}

impl<Http> GrpcHttpMultiplexLayer<Http> {
    pub fn new_with_http(http: Http) -> Self {
        Self { http }
    }
}

impl<Grpc, Http> Layer<Grpc> for GrpcHttpMultiplexLayer<Http>
where Http: Clone {
    type Service = GrpcHttpMultiplexService<Grpc, Http>;

    fn layer(&self, grpc: Grpc) -> Self::Service {
        GrpcHttpMultiplexService {
            grpc,
            http: self.http.clone(),
            grpc_ready: false,
            http_ready: false,
        }
    }
}

fn is_grpc_request<Body>(request: &Request<Body>) -> bool {
    request.headers()
        .get(CONTENT_TYPE)
        .map(|content_type| content_type.as_bytes())
        .filter(|content_type| content_type.starts_with(b"application/grpc"))
        .is_some()
}
