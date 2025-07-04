use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Field, Fields, Item, Token, parse_quote, parse2, punctuated::Punctuated, spanned::Spanned as _,
};

use crate::{
    options::Options,
    parse::{AmassApply, AmassFrom, AmassVariant},
    variant_action::VariantAction,
};

pub(crate) fn amass_apply(attr: TokenStream) -> syn::Result<TokenStream> {
    let span = attr.span();

    let AmassApply {
        mut common,
        generic_arguments,
        telety_item,
    } = parse2(attr)?;

    let telety = telety::Telety::new(&telety_item)?;
    let mut apply_args_visitor = telety.generics_visitor(&generic_arguments.args)?;

    let options = match Options::from_attrs(telety.attributes()) {
        Ok(options) => options,
        Err(_) => return Ok(TokenStream::new()),
    };

    let Item::Enum(amass_enum) = telety.item() else {
        return Err(syn::Error::new(
            telety.item().span(),
            "Only enums can be amassed",
        ));
    };

    let mut generated_items = vec![];

    let primary_path = telety
        .alias_map()
        .get_self()
        .expect("Self must be aliased")
        .to_macro_path();

    for variant in amass_enum.variants.iter() {
        let variant_action = VariantAction::from_attrs(&variant.attrs)?
            .unwrap_or(options.default.unwrap_or_default());

        let empty_fields = Punctuated::<Field, Token![,]>::new();
        let mut field_iter = match &variant.fields {
            Fields::Named(fields) => fields.named.iter(),
            Fields::Unnamed(fields) => fields.unnamed.iter(),
            Fields::Unit => empty_fields.iter(),
        };

        let (Some(single_field), None) = (field_iter.next(), field_iter.next()) else {
            if variant_action == VariantAction::Force {
                return Err(syn::Error::new(
                    variant.span(),
                    "Variant must be a single field variant",
                ));
            } else {
                continue;
            }
        };

        if let Some(amass_variant) = AmassVariant::from_field(&primary_path, variant, single_field)
        {
            let type_path = if let syn::Type::Path(type_path) = &single_field.ty {
                Some(type_path)
            } else {
                None
            };

            let variant_action = VariantAction::from_attrs(&variant.attrs)?
                .unwrap_or(options.default.unwrap_or_default());
            if variant_action != VariantAction::Ignore {
                if let Some(type_path) = type_path {
                    // TODO this probably panics if user incorrectly uses a type parameter as a variant
                    let alias = telety
                        .alias_map()
                        .get_alias(type_path)?
                        .expect("type must have an alias");
                    let ty_macro_path = alias.to_macro_path();
                    let mut args = alias.generic_arguments().cloned();
                    if let Some(args) = &mut args {
                        directed_visit::visit_mut(
                            &mut directed_visit::syn::direct::FullDefault,
                            &mut apply_args_visitor,
                            args,
                        );
                        directed_visit::visit_mut(
                            &mut directed_visit::syn::direct::FullDefault,
                            &mut telety.alias_map().visitor(),
                            args,
                        );
                    }

                    // TODO should be in a drop guard
                    common.push_variant(amass_variant);

                    let amass_from = AmassFrom {
                        common,
                        leaf_type: parse_quote!(#ty_macro_path #args),
                    };

                    generated_items.push(quote_spanned!(span =>
                        ::amass::__private::amass_from!(#amass_from);
                    ));

                    common = amass_from.common;

                    if variant_action == VariantAction::Deep
                        || variant_action == VariantAction::Force
                    {
                        let fallback = if variant_action == VariantAction::Force {
                            quote_spanned!( span =>
                                ::amass::__private::require_telety_error!();
                            )
                        } else {
                            TokenStream::new()
                        };

                        let ty_macro_path = alias.to_macro_path();

                        let needle = Ident::new("__amass_apply_needle", span);

                        let arg_contents = args.as_ref().map(|a| &a.args);
                        let mut amass_apply_macro = telety::v1::TY.apply(
                            ty_macro_path.clone(),
                            needle.clone(),
                            quote_spanned!(span => {
                                ::amass::__private::amass_apply!(#common <#arg_contents> #needle);
                            }))
                            .with_fallback(fallback);

                        if let Some(telety_path) = telety.options().telety_path.as_ref() {
                            amass_apply_macro =
                                amass_apply_macro.with_telety_path(telety_path.clone());
                        }

                        generated_items.push(amass_apply_macro.into_token_stream());
                    }

                    common.pop_variant();
                } else if variant_action == VariantAction::Force {
                    return Err(syn::Error::new(
                        variant.fields.span(),
                        "Non-path types cannot have deep impls",
                    ));
                }
            }
        }
    }

    Ok(quote!(#(#generated_items)*))
}
