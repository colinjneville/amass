use proc_macro2::TokenStream;
use syn::parse2;

use crate::parse::AmassFrom;

pub(crate) fn amass_from(attr: TokenStream) -> syn::Result<TokenStream> {
    let args: AmassFrom = parse2(attr)?;

    Ok(args.generate())
}
