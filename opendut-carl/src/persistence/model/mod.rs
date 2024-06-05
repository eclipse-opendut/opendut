use std::marker::PhantomData;

mod peer_descriptor;


trait Persistable<Model>: From<Model>
where
    Model: TryFrom<Self, Error=PersistableConversionError<Self, Model>>
{ }

#[derive(Debug, thiserror::Error)]
#[error("Could not convert persistable from `{}` to `{}`:\n  {cause}", std::any::type_name::<From>(), std::any::type_name::<To>())]
struct PersistableConversionError<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
    #[source] cause: Box<dyn std::error::Error + Send + Sync>,
}
impl<From, To> PersistableConversionError<From, To> {
    pub fn new(cause: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            _from: PhantomData,
            _to: PhantomData,
            cause,
        }
    }
}
