use std::any::Any;
use crate::prelude::ResourceRef;
use crate::resource::marshalling::{MarshalError, MarshallerIdentifier, UnmarshalError};
use crate::resource::Resource;

pub trait GenericResource {

    fn resource_ref(&self) -> &dyn GenericResourceRef;

    fn resource_ref_mut(&self) -> &mut dyn GenericResourceRef;
}

impl <R: Resource + Any> GenericResource for R {

    fn resource_ref(&self) -> &dyn GenericResourceRef {
        todo!()//&self.resource_ref()
    }

    fn resource_ref_mut(&self) -> &mut dyn GenericResourceRef {
        todo!()
    }
}

impl <R, RR> GenericResourceRef for R
where
    R: Resource<ResourceRef=RR>,
    RR: ResourceRef<R>
{

}

pub trait GenericResourceRef {

}

pub trait GenericMarshaller: Sync + Send {

    fn identifier(&self) -> MarshallerIdentifier where Self: Sized;

    fn marshal(&self, resource: &dyn GenericResource) -> Result<Vec<u8>, MarshalError>;

    fn unmarshal(&self, resource: &[u8]) -> Result<Box<dyn GenericResource>, UnmarshalError>;
}
