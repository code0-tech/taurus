//! Expands the function-like `taurus_macros::flow_type! { ... }` macro. Flow
//! types carry no handler body, like data types -- just metadata.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::{AttrArgs, optional_string, translation_vec};

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let args = AttrArgs::parse(input)?;

    let identifier = args.required_string("identifier")?;
    let module = args.required_string("module")?;
    let name = translation_vec(&args.translations("name")?);
    let description = translation_vec(&args.translations("description")?);
    let documentation = translation_vec(&args.translations("documentation")?);
    let display_message = translation_vec(&args.translations("display_message")?);
    let alias = translation_vec(&args.translations("alias")?);
    let editable = args.flag("editable");
    let display_icon = optional_string(args.string("display_icon")?);
    let linked: Vec<String> = args.string_array("linked_data_type_identifiers")?;
    let signature = args.string("signature")?.unwrap_or_default();

    let meta_fn_ident = format_ident!(
        "__taurus_flow_type_meta_{}",
        identifier
            .to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
    );

    Ok(quote! {
        #[doc(hidden)]
        fn #meta_fn_ident() -> crate::meta::FlowTypeMeta {
            crate::meta::FlowTypeMeta {
                identifier: #identifier,
                module: #module,
                name: #name,
                description: #description,
                documentation: #documentation,
                display_message: #display_message,
                alias: #alias,
                editable: #editable,
                display_icon: #display_icon,
                linked_data_type_identifiers: vec![#(#linked),*],
                signature: #signature,
            }
        }

        ::inventory::submit! {
            crate::meta::FlowTypeRegistration(#meta_fn_ident)
        }
    })
}
