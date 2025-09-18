mod parse;
mod codegen;
mod args;
mod viper_attributes;

use crate::parse::ParsedInput;
use quote::quote;
use syn::visit_mut::{VisitMut};

use crate::args::{ParsedPyGenAttributes};
use syn::{parse_macro_input, Attribute, File, FnArg, ImplItemFn, Meta};


#[proc_macro_attribute]
pub fn pygen(attributes: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let parsed_attributes = parse_macro_input!(attributes as ParsedPyGenAttributes);

    let parsed_input = {
        let item = Clone::clone(&item);
        syn::parse_macro_input!(item as ParsedInput)
    };

    let mut cleaned_output: File = syn::parse(item).unwrap();
    RemovePygenAttributes.visit_file_mut(&mut cleaned_output);

    let code = codegen::codegen(&parsed_attributes, &parsed_input);

    let struct_name = &parsed_input.struct_name;

    quote! {
        #cleaned_output
        impl #struct_name {
            pub const GENERATED_PYTHON_CODE: &'static str = #code;
        }
    }.into()
}

struct RemovePygenAttributes;

impl VisitMut for RemovePygenAttributes {

    fn visit_fn_arg_mut(&mut self, item: &mut FnArg) {
        match item {
            FnArg::Receiver(_) => {}
            FnArg::Typed(pat) => {
                Self::retain_other_attributes(&mut pat.attrs);
            }
        }
        syn::visit_mut::visit_fn_arg_mut(self, item);
    }

    fn visit_impl_item_fn_mut(&mut self, item: &mut ImplItemFn) {
        Self::retain_other_attributes(&mut item.attrs);
        syn::visit_mut::visit_impl_item_fn_mut(self, item);   
    }
}

impl RemovePygenAttributes {
    fn retain_other_attributes(attr: &mut Vec<Attribute>) {
        attr.retain(|attr| {
            match &attr.meta {
                Meta::List(attr) => {
                    !attr.path.is_ident("viper")
                }
                _ => {
                    true
                }
            }
        });
    }
}
