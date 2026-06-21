//! fre-macros — proc-macro crate for the Fre SRPG project.
//!
//! Provides three derive macros that eliminate boilerplate:
//!
//! - `#[derive(DomainEvent)]` — generates `impl DomainEvent for TypeName {}`
//! - `#[derive(RuleFailure)]` — generates `impl RuleFailure for TypeName`
//!   with per-variant error codes from `#[code = "..."]` attributes.
//! - `#[derive(DefinitionType)]` — generates `impl DefinitionType for TypeName`
//!   with bucket name and extension from struct-level attributes,
//!   and optional custom validation via `#[validate(with = "...")]`.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error, Expr, Lit, Meta};

// ============================================================================
// #[derive(DomainEvent)]
// ============================================================================

/// Derive `DomainEvent` marker trait for a domain event type.
#[proc_macro_derive(DomainEvent)]
pub fn derive_domain_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let gen = quote! {
        impl crate::shared::diagnostics::DomainEvent for #name {}
    };
    gen.into()
}

// ============================================================================
// #[derive(RuleFailure)]
// ============================================================================

/// Derive `RuleFailure` for an error-code enum.
///
/// Each variant should carry a `#[code = "..."]` attribute with the
/// machine-readable error code.
#[proc_macro_derive(RuleFailure, attributes(code))]
pub fn derive_rule_failure(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let match_arms = match &input.data {
        syn::Data::Enum(data) => data.variants.iter().map(|variant| {
            let var_ident = &variant.ident;
            let code_str = extract_code_attr(&variant.attrs)
                .unwrap_or_else(|| var_ident.to_string().to_uppercase());

            let pattern = match &variant.fields {
                syn::Fields::Named(fields) => {
                    let field_idents: Vec<_> = fields.named.iter().map(|f| {
                        f.ident.as_ref().expect("named field without ident")
                    }).collect();
                    quote! { Self::#var_ident { #(#field_idents),* } }
                }
                syn::Fields::Unnamed(fields) => {
                    let underscores: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| syn::Ident::new(&format!("__{}", i), var_ident.span()))
                        .collect();
                    quote! { Self::#var_ident(#(#underscores),*) }
                }
                syn::Fields::Unit => quote! { Self::#var_ident },
            };

            quote! { #pattern => #code_str }
        }).collect::<Vec<_>>(),
        syn::Data::Struct(_) => {
            let code_str = extract_code_attr(&input.attrs)
                .unwrap_or_else(|| name.to_string().to_uppercase());
            vec![quote! { _ => #code_str }]
        }
        syn::Data::Union(_) => {
            return Error::new_spanned(name, "RuleFailure cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let gen = quote! {
        impl crate::shared::traits::RuleFailure for #name {
            fn code(&self) -> &'static str {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    gen.into()
}

// ============================================================================
// #[derive(DefinitionType)]
// ============================================================================

/// Derive `DefinitionType` for a Definition struct.
///
/// Required attribute:
/// - `#[bucket = "..."]` — bucket name in the definition registry.
///
/// Optional attributes:
/// - `#[extension = "..."]` — file extension (default `"ron"`).
/// - `#[validate(with = "path::to::fn")]` — custom validation function.
#[proc_macro_derive(DefinitionType, attributes(bucket, extension, validate))]
pub fn derive_definition_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let bucket_str = extract_string_attr(&input.attrs, "bucket")
        .unwrap_or_else(|| {
            let s = name.to_string();
            if s.ends_with("Def") {
                let base = &s[..s.len() - 3];
                base.to_lowercase() + "s"
            } else {
                s.to_lowercase()
            }
        });

    let extension_str = extract_string_attr(&input.attrs, "extension")
        .unwrap_or_else(|| "ron".to_string());

    let validate_fn = extract_validate_with(&input.attrs);

    let gen = if let Some(fn_path) = validate_fn {
        quote! {
            impl crate::content::loading::DefinitionType for #name {
                const BUCKET_NAME: &'static str = #bucket_str;
                const EXTENSION: &'static str = #extension_str;

                fn validate(&self) -> Result<(), crate::content::loading::ValidationError> {
                    #fn_path(self)
                }
            }
        }
    } else {
        quote! {
            impl crate::content::loading::DefinitionType for #name {
                const BUCKET_NAME: &'static str = #bucket_str;
                const EXTENSION: &'static str = #extension_str;

                fn validate(&self) -> Result<(), crate::content::loading::ValidationError> {
                    Ok(())
                }
            }
        }
    };

    gen.into()
}

// ============================================================================
// Helper functions
// ============================================================================

/// Extract the string value from a `#[key = "..."]` attribute (syn 2.0 API).
fn extract_string_attr(attrs: &[syn::Attribute], key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(key) {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(lit) = &expr_lit.lit {
                        return Some(lit.value());
                    }
                }
            }
        }
    }
    None
}

/// Extract the string value from a `#[code = "..."]` variant attribute.
fn extract_code_attr(attrs: &[syn::Attribute]) -> Option<String> {
    extract_string_attr(attrs, "code")
}

/// Extract the function path from `#[validate(with = "...")]`.
fn extract_validate_with(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("validate") {
            if let Meta::List(meta_list) = &attr.meta {
                return parse_validate_with_tokens(&meta_list.tokens);
            }
        }
    }
    None
}

/// Parse the inner tokens of `#[validate(with = "path::to::fn")]`.
fn parse_validate_with_tokens(tokens: &proc_macro2::TokenStream) -> Option<String> {
    use syn::parse::{Parse, ParseStream};

    struct ValidateArgs {
        fn_path: String,
    }

    impl Parse for ValidateArgs {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let ident: syn::Ident = input.parse()?;
            if ident != "with" {
                return Err(input.error("expected `with`"));
            }
            let _eq: syn::Token![=] = input.parse()?;
            let path: syn::LitStr = input.parse()?;
            Ok(ValidateArgs {
                fn_path: path.value(),
            })
        }
    }

    syn::parse2::<ValidateArgs>(tokens.clone())
        .ok()
        .map(|args| args.fn_path)
}
