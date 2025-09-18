use syn::Meta;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

#[derive(Debug, Default)]
pub struct ViperAttributes {
    pub name: Option<String>,
    pub default: Option<String>,
    pub skip: Option<bool>
}

impl ViperAttributes {

    pub fn skip(&self) -> bool {
        self.skip.unwrap_or(false)
    }

    pub fn peek(meta: &Meta) -> bool {
        if let Meta::List(meta_list) = meta {
            meta_list.path.is_ident("viper")
        } else {
            false
        }
    }

    pub fn parse(meta: Meta) -> syn::Result<ViperAttributes> {
        let span = meta.span();

        if let Meta::List(meta_list) = meta {
            let path = &meta_list.path;

            if meta_list.path.is_ident("viper") {
                let tokens = meta_list.tokens;
                syn::parse2::<ViperAttributes>(tokens)
            } else {
                Err(syn::Error::new(span, format!("Expected attribute `viper`, found `{path:?}`")))
            }
        } else {
            Err(syn::Error::new(span, format!("Expected a list, found `{meta:?}`")))
        }
    }
}

impl Parse for ViperAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let mut viper_attribute = ViperAttributes::default();


        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            let ident = input.parse::<syn::Ident>()?;
            let ident_name = ident.to_string();
            match ident_name.as_str() {
                "name" => {
                    let value = parse_attribute_value(input)?;
                    viper_attribute.name = Some(value);
                    Ok(viper_attribute)
                }
                "default" => {
                    let value = parse_attribute_value(input)?;
                    viper_attribute.default = Some(value);
                    Ok(viper_attribute)
                }
                "skip" => {
                    viper_attribute.skip = Some(true);
                    Ok(viper_attribute)
                }
                &_ => {
                    Err(syn::Error::new(ident.span(), format!("Unknown argument: {ident_name}")))
                }
            }
        }
        else {
            Err(lookahead.error())
        }
    }
}

fn parse_attribute_value(input: ParseStream) -> syn::Result<String> {

    let lookahead = input.lookahead1();
    if lookahead.peek(syn::Token![=]) {
        input.parse::<syn::Token![=]>()?;
        let value = input.parse::<syn::LitStr>()?;
        Ok(value.value())
    }
    else {
        Err(lookahead.error())
    }
}
