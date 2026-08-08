//! Expands `#[taurus_macros::runtime_function(...)]`. This macro is only
//! ever invoked from inside `taurus-core` itself (its handler modules), so
//! the generated code can and does refer to `crate::...` / `::inventory`
//! directly rather than needing a re-export indirection layer.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemFn;

use crate::parse::{AttrArgs, optional_string, take_repeated, translation_vec};

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut item_fn: ItemFn = syn::parse2(item)?;
    let args = AttrArgs::parse(attr)?;

    let parameters = take_repeated(&mut item_fn.attrs, "parameter")?
        .iter()
        .map(parameter_tokens)
        .collect::<syn::Result<Vec<_>>>()?;
    let param_count = parameters.len();

    let identifier = args.required_string("identifier")?;
    let module = args.required_string("module")?;
    let signature = args.required_string("signature")?;
    let name = translation_vec(&args.translations("name")?);
    let description = translation_vec(&args.translations("description")?);
    let documentation = translation_vec(&args.translations("documentation")?);
    let display_message = translation_vec(&args.translations("display_message")?);
    let alias = translation_vec(&args.translations("alias")?);
    let display_icon = optional_string(args.string("display_icon")?);
    let throws_error = args.flag("throws_error");
    let linked: Vec<String> = args.string_array("linked_data_type_identifiers")?;

    let param_modes = args.ident_array("param_modes")?;
    if !param_modes.is_empty() && param_modes.len() != param_count {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "`param_modes` has {} entries but {param_count} `#[parameter(...)]` are declared",
                param_modes.len(),
            ),
        ));
    }

    let fn_ident = &item_fn.sig.ident;
    let dispatch_entry = if param_modes.is_empty() {
        quote! {
            crate::handler::registry::FunctionRegistration::eager(
                #identifier, #fn_ident, #param_count as u8,
            )
        }
    } else {
        quote! {
            crate::handler::registry::FunctionRegistration::modes(
                #identifier, #fn_ident,
                &[#(crate::handler::argument::ParameterNode::#param_modes),*],
            )
        }
    };

    let meta_fn_ident = format_ident!("__taurus_meta_{}", fn_ident);

    Ok(quote! {
        #item_fn

        ::inventory::submit! {
            #dispatch_entry
        }

        #[doc(hidden)]
        fn #meta_fn_ident() -> crate::meta::RuntimeFunctionMeta {
            crate::meta::RuntimeFunctionMeta {
                identifier: #identifier,
                module: #module,
                signature: #signature,
                name: #name,
                description: #description,
                documentation: #documentation,
                display_message: #display_message,
                alias: #alias,
                display_icon: #display_icon,
                throws_error: #throws_error,
                parameters: vec![#(#parameters),*],
                linked_data_type_identifiers: vec![#(#linked),*],
            }
        }

        ::inventory::submit! {
            crate::meta::MetaRegistration(#meta_fn_ident)
        }
    })
}

fn parameter_tokens(args: &AttrArgs) -> syn::Result<TokenStream> {
    let runtime_name = args.required_string("runtime_name")?;
    let name = translation_vec(&args.translations("name")?);
    let description = translation_vec(&args.translations("description")?);
    let documentation = translation_vec(&args.translations("documentation")?);

    Ok(quote! {
        crate::meta::ParameterMeta {
            runtime_name: #runtime_name,
            name: #name,
            description: #description,
            documentation: #documentation,
        }
    })
}
