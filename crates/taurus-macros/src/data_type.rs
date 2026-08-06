//! Expands the function-like `taurus_macros::data_type! { ... }` macro. Data
//! types carry no handler body (they're pure metadata, structurally a
//! hand-authored type string), so unlike `#[runtime_function]` this doesn't
//! attach to an existing item -- it just declares a hidden meta fn and
//! registers it.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::{AttrArgs, translation_vec};

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let args = AttrArgs::parse(input)?;

    let identifier = args.required_string("identifier")?;
    let module = args.required_string("module")?;
    let name = translation_vec(&args.translations("name")?);
    let display_message = translation_vec(&args.translations("display_message")?);
    let alias = translation_vec(&args.translations("alias")?);
    let type_string = args.required_string("type_string")?;
    let generic_keys: Vec<String> = args.string_array("generic_keys")?;
    let linked: Vec<String> = args.string_array("linked_data_type_identifiers")?;

    let regex = args.string("regex")?;
    let number_range_from = args.int("number_range_from")?;
    let number_range_to = args.int("number_range_to")?;
    let mut rules = Vec::new();
    if let Some(pattern) = regex {
        rules.push(quote! {
            ::tucana::shared::DefinitionDataTypeRule {
                config: Some(::tucana::shared::definition_data_type_rule::Config::Regex(
                    ::tucana::shared::DataTypeRegexRuleConfig { pattern: #pattern.to_string() },
                )),
            }
        });
    }
    match (number_range_from, number_range_to) {
        (Some(from), Some(to)) => rules.push(quote! {
            ::tucana::shared::DefinitionDataTypeRule {
                config: Some(::tucana::shared::definition_data_type_rule::Config::NumberRange(
                    ::tucana::shared::DataTypeNumberRangeRuleConfig { from: #from, to: #to, steps: None },
                )),
            }
        }),
        (None, None) => {}
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`number_range_from` and `number_range_to` must be given together",
            ));
        }
    }

    let meta_fn_ident = format_ident!(
        "__taurus_data_type_meta_{}",
        identifier
            .to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
    );

    Ok(quote! {
        #[doc(hidden)]
        fn #meta_fn_ident() -> crate::meta::DataTypeMeta {
            crate::meta::DataTypeMeta {
                identifier: #identifier,
                module: #module,
                name: #name,
                display_message: #display_message,
                alias: #alias,
                generic_keys: vec![#(#generic_keys),*],
                type_string: #type_string,
                linked_data_type_identifiers: vec![#(#linked),*],
                rules: vec![#(#rules),*],
            }
        }

        ::inventory::submit! {
            crate::meta::DataTypeRegistration(#meta_fn_ident)
        }
    })
}
