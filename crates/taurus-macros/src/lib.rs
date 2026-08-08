//! Macros that replace `taurus`'s `definitions/*.json` files: each
//! implemented runtime function, data type, and module declares its own
//! metadata inline and self-registers into `taurus-core`'s `inventory`
//! registries at compile time, instead of that metadata being hand-kept in
//! sync with a separate JSON tree.
//!
//! These macros are only ever invoked from within `taurus-core` (its
//! `runtime::functions::*` handler modules), so the code they generate
//! refers to `crate::...` directly rather than through a re-export
//! indirection layer -- see `taurus-core/src/meta.rs` for the types it
//! constructs and `taurus-core/src/handler/registry.rs` for the dispatch
//! entry it constructs.

mod data_type;
mod module;
mod parse;
mod runtime_function;

use proc_macro::TokenStream;

fn run_attr(
    f: impl FnOnce(
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ) -> syn::Result<proc_macro2::TokenStream>,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    f(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn run_fnlike(
    f: impl FnOnce(proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream>,
    input: TokenStream,
) -> TokenStream {
    f(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Declares a runtime-invocable handler `fn`: generates its dispatch-table
/// entry and its `RuntimeFunctionMeta`, and self-registers both.
#[proc_macro_attribute]
pub fn runtime_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    run_attr(runtime_function::expand, attr, item)
}

/// Declares a data type (was `data_types/*.json`).
#[proc_macro]
pub fn data_type(input: TokenStream) -> TokenStream {
    run_fnlike(data_type::expand, input)
}

/// Declares a module (was `module.json`). Exactly one per feature file.
#[proc_macro]
pub fn module(input: TokenStream) -> TokenStream {
    run_fnlike(module::expand, input)
}
