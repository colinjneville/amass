mod impl_type;
mod options;
mod parse;
mod proc_impl;
mod syn_util;
mod variant_action;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn amass(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (Ok(ts) | Err(ts)) =
        proc_impl::amass(attr.into(), item.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro_attribute]
pub fn amass_telety(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (Ok(ts) | Err(ts)) =
        proc_impl::amass_telety(attr.into(), item.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
#[doc(hidden)]
pub fn amass_from(attr: TokenStream) -> TokenStream {
    let (Ok(ts) | Err(ts)) =
        proc_impl::amass_from(attr.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
#[doc(hidden)]
pub fn amass_apply(attr: TokenStream) -> TokenStream {
    let (Ok(ts) | Err(ts)) =
        proc_impl::amass_apply(attr.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}
