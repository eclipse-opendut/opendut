extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use proc_macro::TokenStream;

#[proc_macro_derive(ResourceRef)]
pub fn derive_resource_ref(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let gen = quote! {
        impl opendut_resources::resource::versioning::Versioned for #name {
            fn current_hash(&self) -> &opendut_resources::resource::versioning::RevisionHash {
                &self.current_hash
            }
            fn parent_hash(&self) -> &opendut_resources::resource::versioning::RevisionHash {
                &self.parent_hash
            }
            // fn derived_revision(&self) -> Self::Derived {
            //     let mut derived = Clone::clone(self);
            //     <Self as opendut_resources::resource::versioning::VersionedMut>::reset_revision(&mut derived, opendut_resources::resource::versioning::ROOT_REVISION_HASH, self.current_hash);
            //     derived
            // }
        }
        impl opendut_resources::resource::versioning::VersionedMut for #name {
            fn current_hash_mut(&mut self) -> &mut opendut_resources::resource::versioning::RevisionHash {
                &mut self.current_hash
            }
            fn parent_hash_mut(&mut self) -> &mut opendut_resources::resource::versioning::RevisionHash {
                &mut self.parent_hash
            }
        }
        impl opendut_resources::resource::versioning::ToRevision for #name {
            fn revision(&self) -> opendut_resources::resource::versioning::Revision {
                opendut_resources::resource::versioning::Revision::new(self.current_hash, self.parent_hash)
            }
        }
        impl opendut_resources::resource::versioning::BorrowRevision for #name {
            fn borrow_revision(&self) -> opendut_resources::resource::versioning::BorrowedRevision<Self> {
                opendut_resources::resource::versioning::BorrowedRevision::new(self)
            }
        }
        impl opendut_resources::resource::versioning::BorrowMutRevision for #name {
            fn borrow_mut_revision(&mut self) -> opendut_resources::resource::versioning::BorrowedMutRevision<Self> {
                opendut_resources::resource::versioning::BorrowedMutRevision::new(self)
            }
        }
    };
    gen.into()
}


#[proc_macro_derive(Marshaller)]
pub fn derive_marshaller(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let gen = quote! {
        impl opendut_resources::resource::generic::GenericMarshaller for #name {

            fn identifier(&self) -> opendut_resources::resource::marshalling::MarshallerIdentifier where Self: Sized {
                Self::IDENTIFIER
            }

            fn marshal(&self, resource: &dyn opendut_resources::resource::generic::GenericResource) -> Result<Vec<u8>, opendut_resources::resource::marshalling::MarshalError> {
                let resource = resource.downcast_ref::<&AwesomeResource>();
                opendut_resources::resource::marshalling::Marshaller::marshal(self, resource)
            }

            fn unmarshal(&self, resource: &[u8]) -> Result<Box<dyn opendut_resources::resource::generic::GenericResource>, opendut_resources::resource::marshalling::UnmarshalError> {
                opendut_resources::resource::marshalling::Marshaller::unmarshal(self, resource)
            }
        }
    };
    gen.into()
}
