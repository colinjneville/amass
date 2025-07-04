use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned as _,
    token::Bracket,
    AngleBracketedGenericArguments, Field, Generics, Ident, Item, Path, PathArguments, PathSegment,
    Token, Type, Variant,
};

pub(crate) struct AmassFrom {
    pub common: AmassCommon,
    pub leaf_type: syn::TypePath,
}

impl Parse for AmassFrom {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            common: input.parse()?,
            leaf_type: input.parse()?,
        })
    }
}

impl ToTokens for AmassFrom {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { common, leaf_type } = self;
        common.to_tokens(tokens);
        leaf_type.to_tokens(tokens);
    }
}

pub(crate) struct AmassVariant {
    path: Path,
    named_field: Option<(Token![.], Ident)>,
}

impl AmassVariant {
    pub fn from_field(enum_type: &Path, variant: &Variant, field: &Field) -> Option<Self> {
        let named_field = field
            .ident
            .as_ref()
            .map(|i| (Default::default(), i.clone()));

        let mut path = enum_type.clone();
        for segment in path.segments.iter_mut() {
            segment.arguments = PathArguments::None;
        }
        path.segments.push(PathSegment {
            ident: variant.ident.clone(),
            arguments: PathArguments::None,
        });

        Some(Self { path, named_field })
    }
}

impl Parse for AmassVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        let field = if let Some(dot) = input.parse()? {
            Some((dot, input.parse()?))
        } else {
            None
        };

        Ok(Self {
            path,
            named_field: field,
        })
    }
}

impl ToTokens for AmassVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
        if let Some((dot, field)) = &self.named_field {
            dot.to_tokens(tokens);
            field.to_tokens(tokens);
        }
    }
}

pub(crate) struct AmassCommon {
    parameters: Generics,
    root_type: Type,
    variants_bracket: Bracket,
    // [
    variants: Punctuated<AmassVariant, Token![,]>,
    // ]
}

impl AmassCommon {
    pub fn new(parameters: Generics, root_type: Type) -> Self {
        Self {
            parameters,
            root_type,
            variants_bracket: Default::default(),
            variants: Punctuated::new(),
        }
    }

    pub fn push_variant(&mut self, variant: AmassVariant) {
        self.variants.push(variant);
    }

    pub fn pop_variant(&mut self) {
        self.variants.pop().expect("unbalanced variant stack");
    }
}

impl Parse for AmassCommon {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            parameters: input.parse()?,
            root_type: input.parse()?,
            variants_bracket: bracketed!(content in input),
            variants: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl ToTokens for AmassCommon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            parameters,
            root_type,
            variants_bracket,
            variants,
        } = self;
        parameters.to_tokens(tokens);
        root_type.to_tokens(tokens);
        variants_bracket.surround(tokens, |ts| variants.to_tokens(ts));
    }
}

impl AmassFrom {
    pub fn generate(&self) -> TokenStream {
        let Self {
            common:
                AmassCommon {
                    parameters,
                    root_type,
                    variants_bracket: _variants_bracket,
                    variants,
                },
            leaf_type,
        } = self;
        let span = leaf_type.span();

        let (impl_generics, _type_generics, where_clause) = parameters.split_for_impl();

        let mut conversion = quote_spanned!(span => value);
        for variant in variants.into_iter().rev() {
            let path = &variant.path;
            conversion = match &variant.named_field {
                Some((_dot, field)) => quote_spanned!(span => #path { #field: #conversion }),
                None => quote_spanned!(span => #path(#conversion)),
            }
        }

        quote_spanned!(span =>
            impl #impl_generics ::core::convert::From<#leaf_type> for #root_type
            #where_clause {
                fn from(value: #leaf_type) -> Self {
                    #conversion
                }
            }
        )
    }
}

pub(crate) struct AmassApply {
    pub common: AmassCommon,
    pub generic_arguments: AngleBracketedGenericArguments,
    pub telety_item: Item,
}

impl Parse for AmassApply {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            common: input.parse()?,
            generic_arguments: input.parse()?,
            telety_item: input.parse()?,
        })
    }
}

impl ToTokens for AmassApply {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            common,
            generic_arguments,
            telety_item: telety_enum,
        } = self;
        common.to_tokens(tokens);
        generic_arguments.to_tokens(tokens);
        telety_enum.to_tokens(tokens);
    }
}
