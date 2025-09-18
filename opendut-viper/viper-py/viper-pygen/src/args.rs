use syn::parse::{Parse, ParseStream};

pub struct ParsedPyGenAttributes;

impl Parse for ParsedPyGenAttributes {

    fn parse(_input: ParseStream) -> syn::Result<Self> {

        Ok(ParsedPyGenAttributes )
    }
}
