use crate::resource::versioning::{Versioned, VersionedMut};

pub mod versioning;

pub trait Resource : Clone + serde::Serialize {

    type ResourceRef : ResourceRef<Self>;
    type ResourceRefMut : ResourceRefMut<Self>;

    fn resource_ref(&self) -> &Self::ResourceRef;

    fn resource_ref_mut(&mut self) -> &mut Self::ResourceRefMut;

    fn into_resource_ref(self) -> Self::ResourceRef;
}

pub trait ResourceRef<R> : Versioned + Clone
where
    R: Resource
{
    fn uuid(&self) -> &uuid::Uuid;
}

pub trait ResourceRefMut<R> : ResourceRef<R> + VersionedMut
where
    R: Resource
{
    fn uuid_mut(&mut self) -> &mut uuid::Uuid;
}

#[macro_export]
/// Implements traits required for a struct to be a [`Resource`] and it's corresponding [`ResourceRef`].
///
/// This macro takes as first parameter the struct designated as [`Resource`]
/// and as second parameter the struct designated as [`ResourceRef`].
///
/// ## Requirements
///
/// The struct designated as [`Resource`] has to fulfill the following requirements:
/// - contains a field `id` with the type designated as [`ResourceRef`].
/// - implements the super traits of the [`Resource`] trait.
///
/// The struct designated as [`ResourceRef`] has to fulfill the following requirements:
/// - contains a field `id` with type [`uuid::Uuid`].
/// - implements the [`RevisionRef`] trait, and it's  super traits.
///
/// ## Example
///
/// ```
/// use serde::Serialize;
/// use uuid::Uuid;
/// use opendut_resources::prelude::*;
///
/// #[derive(Clone, Serialize)]
/// struct MyResource {
///     id: MyResourceId
/// }
///
/// #[derive(Clone, ResourceRef, Serialize)]
/// struct MyResourceId {
///     id: Uuid,
///     current_hash: RevisionHash,
///     parent_hash: RevisionHash
/// }
///
/// resource!(MyResource, MyResourceId);
/// ```
macro_rules! resource {

    ( $resource:ident, $resource_ref:ident ) => {

        impl opendut_resources::resource::Resource for $resource {

            type ResourceRef = $resource_ref;
            type ResourceRefMut = $resource_ref;

            fn resource_ref(&self) -> &$resource_ref {
                &self.id
            }

            fn resource_ref_mut(&mut self) -> &mut $resource_ref {
                &mut self.id
            }

            fn into_resource_ref(self) -> $resource_ref {
                self.id
            }
        }

        impl opendut_resources::resource::ResourceRef<$resource> for $resource_ref {
            fn uuid(&self) -> &opendut_resources::prelude::Uuid {
                &self.id
            }
        }

        impl opendut_resources::resource::ResourceRefMut<$resource> for $resource_ref {
            fn uuid_mut(&mut self) -> &mut opendut_resources::prelude::Uuid {
                &mut self.id
            }
        }
    };
    (_) => {
         compile_error!("The `resource!` macro requires two parameters. The first one has to be the designated `Resource` struct. The second one has to be the designed `ResourceRef`.");
    };
}
