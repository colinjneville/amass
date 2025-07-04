use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Ident, ItemEnum, Path, Type, TypePath, parse_quote, parse_quote_spanned, parse2,
    spanned::Spanned as _,
    visit_mut::{self, VisitMut as _},
};

use crate::{options::Options, parse::AmassCommon, syn_util};

pub(crate) fn amass(attr_args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attr_span = attr_args.span();

    let _options: Options = parse2(attr_args)?;
    let mut item: ItemEnum = parse2(item)?;

    let (_, type_generics, _) = item.generics.split_for_impl();
    let item_ident = item.ident.clone();
    let item_path: Path = parse_quote!(self::#item_ident);
    let item_path_generics: Path = parse_quote!(#item_path #type_generics);
    let item_type = Type::Path(TypePath {
        qself: None,
        path: item_path_generics,
    });

    let common = AmassCommon::new(item.generics.clone(), item_type);

    let args = syn_util::generic_params_to_arguments(&item.generics);

    let needle = Ident::new("__amass_apply_needle", attr_span);

    let macro_ts = telety::v1::TY
        .apply(
            item_path,
            needle.clone(),
            quote_spanned! { attr_span =>
                ::amass::__private::amass_apply!(
                    #common
                    <#args>
                    #needle
                );
            },
        )
        .with_fallback(quote!(::amass::__private::require_telety_error!();))
        .with_telety_path(parse_quote!(::amass::__private::telety));

    // Strip "helper" attributes because attribute macros still aren't allowed to have them :(
    // https://github.com/rust-lang/rust/issues/65823
    HelperAttributeVisitor.visit_item_enum_mut(&mut item);

    Ok(quote! {
        #item

        #macro_ts
    })
}

struct HelperAttributeVisitor;

impl HelperAttributeVisitor {
    fn is_helper(ident: &Ident) -> bool {
        matches!(ident.to_string().as_str(), "amass_action")
    }

    fn make_noop_attr(attr: &mut Attribute) {
        let span = attr.span();
        *attr = parse_quote_spanned! { span =>
            #[cfg(all())]
        };
    }
}

impl visit_mut::VisitMut for HelperAttributeVisitor {
    fn visit_attribute_mut(&mut self, i: &mut Attribute) {
        if let Some(ident) = i.meta.path().get_ident() {
            if Self::is_helper(ident) {
                Self::make_noop_attr(i);
            }
        }
    }
}
