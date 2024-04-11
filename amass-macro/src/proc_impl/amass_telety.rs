use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2, Path,
};

struct Args {
    telety_path: Path,
    amass_args: TokenStream,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            telety_path: input.parse()?,
            amass_args: input.parse()?,
        })
    }
}

pub(crate) fn amass_telety(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let Args {
        telety_path,
        amass_args,
    } = parse2(attr)?;

    Ok(quote!(
        #[::amass::__private::telety::telety(#telety_path, telety_path = "::amass::__private::telety")]
        #[::amass::amass(#amass_args)]
        #item
    ))
}
