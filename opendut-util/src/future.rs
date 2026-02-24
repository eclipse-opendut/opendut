use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Workaround for compiler bug: <https://github.com/rust-lang/rust/issues/64552>
/// Code adapted from: <https://github.com/rust-lang/rust/issues/64552#issuecomment-604419315>
///
/// Bug started occurring after larger dependency upgrade of the networking stack (http 1.0, hyper 1.0, axum 0.7, tonic 0.12, reqwest 0.12)
pub struct ExplicitSendFutureWrapper<F: Future> {
    future: F,
}

impl<F: Future> From<F> for ExplicitSendFutureWrapper<F> {
    fn from(future: F) -> Self {
        Self { future }
    }
}

unsafe impl<F: Future> Send for ExplicitSendFutureWrapper<F> {}

impl<F: Future> Future for ExplicitSendFutureWrapper<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        unsafe {
            self
                .map_unchecked_mut(|inner_self| &mut inner_self.future)
                .poll(cx)
        }
    }
}
