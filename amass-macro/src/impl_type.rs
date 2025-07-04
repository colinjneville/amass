use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ImplType {
    From,
}

impl Parse for ImplType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ImplTypeParse::parse(input)?.impl_type)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ImplTypeParse {
    impl_type: ImplType,
    span: Span,
}

impl From<ImplTypeParse> for ImplType {
    fn from(value: ImplTypeParse) -> Self {
        value.impl_type
    }
}

impl Parse for ImplTypeParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let ident: Ident = input.parse()?;
        let impl_type = match ident.to_string().as_str() {
            "From" => Ok(ImplType::From),
            _ => Err(syn::Error::new_spanned(ident, "Cannot impl this type")),
        }?;

        Ok(Self { impl_type, span })
    }
}

impl ToTokens for ImplTypeParse {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = match self.impl_type {
            ImplType::From => "From",
        };
        let ident = Ident::new(s, self.span);
        ident.to_tokens(tokens);
    }
}
