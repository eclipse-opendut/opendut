use pin_project_lite::pin_project;
use std::task::Poll;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Elapsed;

#[allow(dead_code)]
pub fn timeout(duration: Duration, future: impl Future) -> Timeout<impl Future> {
    Timeout {
        future,
        time_limit: Instant::now() + duration,
    }
}

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Timeout<F> {
        #[pin]
        future: F,
        time_limit: Instant,
    }
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let time_limit = self.time_limit;
        let this = self.project();
        if let Poll::Ready(value) = this.future.poll(cx) {
            return Poll::Ready(Ok(value));
        }
        if Instant::now() >= time_limit {
            Poll::Ready(Err(Elapsed))
        }
        else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use std::pin::{pin, Pin};
    use std::task::Context;
    use std::thread::sleep;

    #[test]
    fn test_timeout_elapsed() {

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let timeout = pin!(timeout(Duration::from_millis(10), NeverReady));

        sleep(Duration::from_millis(100));

        let result = timeout.poll(&mut cx);

        assert!(matches!(result, Poll::Ready(Err(Elapsed))));
    }

    #[test]
    fn test_timeout_pending() {

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let timeout = pin!(timeout(Duration::from_secs(60), NeverReady));

        let result = timeout.poll(&mut cx);

        assert!(matches!(result, Poll::Pending));
    }

    #[test]
    fn test_timeout_ready() {

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let timeout = pin!(timeout(Duration::from_secs(60), async { 42 }));

        let result = timeout.poll(&mut cx);

        assert!(matches!(result, Poll::Ready(Ok(_))));
    }

    struct NeverReady;
    impl Future for NeverReady {
        type Output = i32;
        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }
}
