
pub mod probe {

    pub fn oneshot<A>() -> oneshot::Probe<A> {
        oneshot::Probe::new()
    }

    pub fn mpsc<A>() -> mpsc::Probe<A> {
        mpsc::Probe::new()
    }

    mod mpsc {
        use std::time::Duration;

        use tokio::sync::mpsc;
        use tokio::time::timeout;

        pub struct Probe<A> {
            sender: mpsc::Sender<A>,
            receiver: mpsc::Receiver<A>
        }

        impl<A> Probe<A> {

            pub fn new() -> Probe<A> {
                let (sender, receiver) = mpsc::channel(1);
                Probe {
                    sender,
                    receiver,
                }
            }

            pub fn sender(&self) -> mpsc::Sender<A> {
                Clone::clone(&self.sender)
            }

            pub async fn receive_message(&mut self) -> A {
                timeout(Duration::from_secs(3), self.receiver.recv()).await
                    .expect("Expect message")
                    .expect("Channel closed")
            }

            pub async fn expect_message<F, T>(&mut self, f: F) -> googletest::Result<T>
            where
                F: FnOnce(A) -> googletest::Result<T>
            {
                let message: A = self.receive_message().await;
                f(message)
            }
        }
    }

    mod oneshot {
        use std::time::Duration;

        use tokio::sync::oneshot;
        use tokio::time::timeout;

        pub struct Probe<A> {
            sender: Option<oneshot::Sender<A>>,
            receiver: oneshot::Receiver<A>
        }

        impl<A> Probe<A> {

            pub fn new() -> Probe<A> {
                let (sender, receiver) = oneshot::channel();
                Probe {
                    sender: Some(sender),
                    receiver,
                }
            }

            pub fn sender(&mut self) -> oneshot::Sender<A> {
                self.sender.take()
                    .unwrap()
            }

            pub async fn receive_message(self) -> A {
                timeout(Duration::from_secs(3), self.receiver).await
                    .expect("Expect message")
                    .expect("Channel closed")
            }

            pub async fn expect_message<F, T>(self, f: F) -> googletest::Result<T>
            where
                F: FnOnce(A) -> googletest::Result<T>
            {
                let message: A = self.receive_message().await;
                f(message)
            }
        }
    }
}
