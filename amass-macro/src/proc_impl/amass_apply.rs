use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned as _,
    visit_mut::VisitMut, Field, Fields, Item, Token,
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

    let primary_path = telety.self_alias().path();

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
            let variant_action = VariantAction::from_attrs(&variant.attrs)?
                .unwrap_or(options.default.unwrap_or_default());
            if variant_action != VariantAction::Ignore {
                let alias = telety
                    .alias_of(&single_field.ty)
                    .expect("variant type should have a mapped alias");

                let mut alias_type = alias.ty();
                apply_args_visitor.visit_type_mut(&mut alias_type);
                telety::visitor::Crateify::new().visit_type_mut(&mut alias_type);

                // TODO should be in a drop guard
                common.push_variant(amass_variant);

                let amass_from = AmassFrom {
                    common,
                    leaf_type: alias_type,
                };

                generated_items.push(quote_spanned!(span =>
                    ::amass::__private::amass_from!(#amass_from);
                ));

                common = amass_from.common;

                if variant_action == VariantAction::Deep || variant_action == VariantAction::Force {
                    let fallback = if variant_action == VariantAction::Force {
                        quote_spanned!( span =>
                            ::amass::__private::require_telety_error!();
                        )
                    } else {
                        TokenStream::new()
                    };

                    let ty_macro_path = &alias.path();

                    let mut new_generic_arguments =
                        alias.aliased_type_arguments().cloned().unwrap_or_default();
                    for argument in new_generic_arguments.iter_mut() {
                        match argument {
                            syn::GenericArgument::Type(ty) => {
                                let alias = telety
                                    .alias_of(ty)
                                    .expect(&format!("argument type {ty:?} should have an alias"));

                                *ty = alias.ty();
                                apply_args_visitor.visit_type_mut(ty);
                            }
                            // TODO support other argument kinds
                            _ => {}
                        }
                    }

                    let needle = Ident::new("__amass_apply_needle", span);

                    let mut amass_apply_macro = telety::v1::TY.apply(
                        ty_macro_path.clone(),
                        needle.clone(),
                        quote_spanned!(span => {
                            ::amass::__private::amass_apply!(#common <#new_generic_arguments> #needle);
                        }))
                        .with_fallback(fallback);

                    if let Some(telety_path) = telety.options().telety_path.as_ref() {
                        amass_apply_macro = amass_apply_macro.with_telety_path(telety_path.clone());
                    }

                    generated_items.push(amass_apply_macro.into_token_stream());
                }

                common.pop_variant();
            }
        }
    }

    Ok(quote!(#(#generated_items)*))
}
