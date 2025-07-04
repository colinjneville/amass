use syn::{
    Attribute, Ident,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned as _,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum VariantAction {
    Ignore,
    Shallow,
    #[default]
    Deep,
    Force,
}

impl VariantAction {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        let mut action = None;
        for attr in attrs {
            if attr.path().is_ident("amass_action") {
                #[allow(
                    clippy::collapsible_if,
                    reason = "separate mutable and immutable clauses"
                )]
                if action
                    .replace(parse2(attr.meta.require_list()?.tokens.clone())?)
                    .is_some()
                {
                    return Err(syn::Error::new(
                        attr.span(),
                        "Only one 'amass' attribute is allowed",
                    ));
                }
            }
        }

        Ok(action)
    }

    pub fn from_ident(ident: &Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "ignore" => Ok(Self::Ignore),
            "shallow" => Ok(Self::Shallow),
            "deep" => Ok(Self::Deep),
            "force" => Ok(Self::Force),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Invalid variant action '{ident}'"),
            )),
        }
    }
}

impl Parse for VariantAction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Self::from_ident(&ident)
    }
}
