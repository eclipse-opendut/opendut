use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{AngleBracketedGenericArguments, BareFnArg, ItemConst, Meta, Token};
use syn::spanned::Spanned;
use syn::token::Pub;
use crate::viper_attributes::ViperAttributes;

pub struct ParsedInput {
    pub struct_name: Ident,
    pub struct_impl: ParsedStructImpl,
    pub _attributes: Vec<ParsedAttribute>,
    pub doc: Vec<String>,
}

pub struct ParsedStructImpl {
    pub functions: Vec<Function>,
}

pub struct Function {
    pub parsed_fn: ParsedFn,
    pub attributes: ViperAttributes,
    pub doc: Vec<String>,
}

pub struct ParsedFn {
    pub fn_name: Ident,
    pub arguments: Vec<Argument>,
    pub return_type: Option<Ident>,
}

pub struct Argument {
    pub arg_name: Ident,
    pub attributes: ViperAttributes,
    pub ty: Option<Ident>,
}

#[derive(Clone, Debug)]
pub struct ParsedAttribute {
    content: TokenStream
}

impl Parse for ParsedInput {

    fn parse(input: ParseStream) -> syn::Result<Self> {

        let mut attributes = Vec::new();

        if !input.peek(Token![impl]) {

            while input.peek(Token![#]) {
                attributes.push(input.parse::<ParsedAttribute>()?);
            }

            if !input.peek(Token![impl]) {
                return Err(syn::Error::new(input.span(), "can only be applied to `impl`"));
            }
        }

        input.parse::<Token![impl]>()?;
        let struct_name = input.parse::<Ident>()?;

        let class_body;
        syn::braced!(class_body in input);

        let struct_impl = class_body.parse::<ParsedStructImpl>()?;

        Ok(ParsedInput { struct_name, struct_impl, doc: find_doc_attributes(&attributes), _attributes: attributes })
    }
}

impl Parse for ParsedStructImpl {

    fn parse(input: ParseStream) -> syn::Result<Self> {

        let mut functions = Vec::new();
        let mut attributes = Vec::new();
        while !input.is_empty() {

            if input.peek(Token![#]) {
                attributes.push(input.parse::<ParsedAttribute>()?);
            }
            else if input.peek(Token![const]) {
                input.parse::<ItemConst>()?;
            }

            else if input.peek(Token![pub]) {
                input.parse::<Pub>()?;
            }
                
            else if input.peek(Token![fn]) {
                let parsed_fn = input.parse::<ParsedFn>()?;
                functions.push(Function { parsed_fn, attributes: find_viper_attributes_or_default(&attributes), doc: find_doc_attributes(&attributes) });
                attributes.clear();
            }
            else { break; }
        }

        Ok(ParsedStructImpl { functions })
    }
}

impl Parse for ParsedFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        input.parse::<Token![fn]>()?;
        let fn_name = input.parse::<Ident>()?;

        let parameter;
        syn::parenthesized!(parameter in input);
        let arguments = parameter.parse_terminated(BareFnArg::parse, Token![,])?
            .into_iter()
            .map(|arg| {

                let span = arg.span();
                let BareFnArg { attrs, name, ty } = arg;

                let arg_type = extract_type_from_option_or_result(&ty);

                let attributes = attrs.into_iter()
                    .find(|attr| ViperAttributes::peek(&attr.meta));

                let viper_attributes = if let Some(attr) = attributes {
                    ViperAttributes::parse(attr.meta)?
                }
                else {
                    ViperAttributes::default()
                };

                let arg_name = match name {
                    None => { Ident::new("self", span) }
                    Some(name) => { name.0 }
                };
                
                Ok(Argument {
                    arg_name,
                    attributes: viper_attributes,
                    ty: arg_type
                })
            }).collect::<Result<_, syn::Error>>()?;

        let mut return_type = None;
        if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            let ty = input.parse::<syn::Type>()?;
            return_type = extract_type_from_option_or_result(&ty);
            if input.peek(Token![<]) {
                input.parse::<AngleBracketedGenericArguments>()?;
            }
        }

        let body;
        syn::braced!(body in input);
        let _: TokenStream = body.parse()?;

        Ok(ParsedFn { fn_name, arguments, return_type })
    }
}

impl Parse for ParsedAttribute {

    fn parse(input: ParseStream) -> syn::Result<Self> {

        input.parse::<Token![#]>()?;

        let content;
        syn::bracketed!(content in input);

        Ok(Self {
            content: content.parse()?,
        })
    }
}

impl ToTokens for ParsedAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.content.to_tokens(tokens);
    }
}

fn find_viper_attributes_or_default(attributes: &Vec<ParsedAttribute>) -> ViperAttributes {

    for attribute in attributes {
        if let Ok(meta) = syn::parse2::<Meta>(attribute.clone().content)
        && ViperAttributes::peek(&meta) {
            return ViperAttributes::parse(meta).unwrap_or_default()
        }
    }
    ViperAttributes::default()
}

fn find_doc_attributes(attributes: &Vec<ParsedAttribute>) -> Vec<String> {

    let mut doc_attributes = Vec::new();

    for attribute in attributes {
        if let Ok(Meta::NameValue(name_value)) = syn::parse2::<Meta>(attribute.clone().content)
        && name_value.path.is_ident("doc")
        && let syn::Expr::Lit(expr_lit) = &name_value.value
        && let syn::Lit::Str(lit_str) = &expr_lit.lit {
            doc_attributes.push(lit_str.value());
        }
    }
    doc_attributes
}

fn extract_type_from_option_or_result(ty: &syn::Type) -> Option<Ident> {
    #[allow(clippy::single_match)]
    match ty {
        syn::Type::Path(path) => {

            let path_segment = path.path.segments.last()?;

            let ident_str = path_segment.ident.to_string();

            match ident_str.as_str() {
                "Option" | "OptionalArg" | "PyResult" | "Result" => {
                    if let syn::PathArguments::AngleBracketed(generics) = &path_segment.arguments
                    && let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = generics.args.last() {
                        inner_path
                            .path
                            .segments
                            .last()
                            .map(|inner_ident| inner_ident.ident.clone())
                    } else {
                        None
                    }
                }
                _ => Some(path_segment.ident.clone()),
            }
        }
        _ => None,
    }
}
