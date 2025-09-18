use std::fmt::Debug;
use futures::{Sink, SinkExt};

#[async_trait::async_trait]
pub trait EventEmitter<E> {
    async fn emit(&mut self, event: E) -> Result<(), EventEmissionError>;
}

pub fn drain<E>() -> impl EventEmitter<E>
where
    E: Send + Sync
{
    DrainEventEmitter::new()
}

pub fn sink<S, I, E>(sink: S) -> impl EventEmitter<I>
where
    S: Sink<I, Error=E> + Unpin + Send + Sync,
    I: Send + Sync,
    E: Debug + Send + Sync,
{
    SinkEventEmitter::new(sink)
}

pub fn fail<E>() -> impl EventEmitter<E>
where
    E: Send + Sync,
{
    FailEventEmitter::new()
}

#[derive(Debug)]
#[non_exhaustive]
pub struct EventEmissionError {
    pub cause: String,
}

struct SinkEventEmitter<S, E>
where
    S: Sink<E> + Unpin,
{
    sink: S,
    _phantom: std::marker::PhantomData<E>,
}

impl <S, E> SinkEventEmitter<S, E>
where
    S: Sink<E> + Unpin,
{
    fn new(sink: S) -> Self {
        Self {
            sink,
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl <S, I, E> EventEmitter<I> for SinkEventEmitter<S, I>
where
    S: Sink<I, Error=E> + Unpin + Send + Sync,
    I: Send + Sync,
    E: Debug + Send + Sync,
{
    async fn emit(&mut self, event: I) -> Result<(), EventEmissionError> {
        self.sink.send(event).await
            .map_err(|e| EventEmissionError { cause: format!("{e:?}") } )
    }
}

#[derive(Default)]
struct DrainEventEmitter<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl <E> DrainEventEmitter<E> {
    fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl <E> EventEmitter<E> for DrainEventEmitter<E>
where
    E: Send + Sync,
{
    async fn emit(&mut self, _event: E) -> Result<(), EventEmissionError> {
        Ok(())
    }
}

#[derive(Default)]
struct FailEventEmitter<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl <E> FailEventEmitter<E> {
    fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl <E> EventEmitter<E> for FailEventEmitter<E>
where
    E: Send + Sync,
{
    async fn emit(&mut self, _event: E) -> Result<(), EventEmissionError> {
        Err(EventEmissionError { cause: String::from("failed") })
    }
}
