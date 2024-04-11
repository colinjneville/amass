use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Attribute, Ident, MetaNameValue, Token,
};

use crate::variant_action::VariantAction;

#[derive(Debug, Default)]
pub(crate) struct Options {
    pub default: Option<VariantAction>,
}

impl Options {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = None;
        for attr in attrs {
            if attr
                .path()
                .segments
                .last()
                .expect("attribute path must not be empty")
                .ident
                == "amass"
            {
                if let Some(options) = match &attr.meta {
                    syn::Meta::Path(_) => Some(Options::default()),
                    syn::Meta::List(meta_list) => Some(parse2(meta_list.tokens.clone())?),
                    syn::Meta::NameValue(_) => {
                        return Err(syn::Error::new(attr.meta.span(), "expected list arguments"))
                    }
                } {
                    if args.replace(options).is_some() {
                        return Err(syn::Error::new(
                            attr.span(),
                            "Only one 'amass' attribute is allowed",
                        ));
                    }
                }
            }
        }

        args.ok_or_else(|| {
            syn::Error::new(
                attrs.first().span(),
                "'amass' attribute not found (aliasing the attribute is not supported)",
            )
        })
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let options: OptionsParse = input.parse()?;

        let mut default = None;

        if let Some((_, named_args)) = &options.args {
            for named_arg in named_args {
                match named_arg.path.get_ident().map(Ident::to_string).as_deref() {
                    Some("default") => {
                        if default
                            .replace(parse2(named_arg.value.to_token_stream())?)
                            .is_some()
                        {
                            return Err(syn::Error::new(
                                named_arg.span(),
                                "parameter appears multiple times",
                            ));
                        }
                    }
                    _ => return Err(syn::Error::new(named_arg.path.span(), "Invalid parameter")),
                }
            }
        }

        Ok(Self { default })
    }
}

pub(crate) struct OptionsParse {
    pub args: Option<(Token![,], Punctuated<MetaNameValue, Token![,]>)>,
}

impl Parse for OptionsParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = if let Some(comma) = input.parse::<Option<Token![,]>>()? {
            let named_args = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;
            Some((comma, named_args))
        } else {
            None
        };

        Ok(Self { args })
    }
}

impl ToTokens for OptionsParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some((comma, args)) = &self.args {
            comma.to_tokens(tokens);
            args.to_tokens(tokens);
        }
    }
}
