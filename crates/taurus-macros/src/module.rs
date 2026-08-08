//! Expands the function-like `taurus_macros::module! { ... }` macro (the
//! Rust-native replacement for a `module.json`). Exactly one per feature
//! file.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::{AttrArgs, translation_vec};

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let args = AttrArgs::parse(input)?;

    let identifier = args.required_string("identifier")?;
    let name = translation_vec(&args.translations("name")?);
    let description = translation_vec(&args.translations("description")?);
    let documentation = args.string("documentation")?.unwrap_or_default();
    let author = args.required_string("author")?;
    let icon = args.required_string("icon")?;
    let version = args.required_string("version")?;

    let meta_fn_ident = format_ident!(
        "__taurus_module_meta_{}",
        identifier
            .to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
    );

    Ok(quote! {
        #[doc(hidden)]
        fn #meta_fn_ident() -> crate::meta::ModuleMeta {
            crate::meta::ModuleMeta {
                identifier: #identifier,
                name: #name,
                description: #description,
                documentation: #documentation,
                author: #author,
                icon: #icon,
                version: #version,
            }
        }

        ::inventory::submit! {
            crate::meta::ModuleRegistration(#meta_fn_ident)
        }
    })
}
