//! Shared parsing for the taurus-macros attribute/declaration macros'
//! argument lists (`identifier = "...", name(en_US = "...")`) and for the
//! repeatable `#[parameter(...)]` helper attribute they read off a handler
//! fn and strip before re-emitting it.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprArray, ExprLit, Lit, Meta, MetaNameValue, Token};

pub struct AttrArgs {
    metas: Vec<Meta>,
}

fn expr_str(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Ok(s.value()),
        other => Err(syn::Error::new_spanned(other, "expected a string literal")),
    }
}

impl AttrArgs {
    pub fn parse(tokens: TokenStream) -> syn::Result<Self> {
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(tokens)?;
        Ok(Self {
            metas: metas.into_iter().collect(),
        })
    }

    fn find(&self, key: &str) -> Option<&Meta> {
        self.metas.iter().find(|m| m.path().is_ident(key))
    }

    /// `key = "..."`.
    pub fn string(&self, key: &str) -> syn::Result<Option<String>> {
        match self.find(key) {
            None => Ok(None),
            Some(Meta::NameValue(MetaNameValue { value, .. })) => Ok(Some(expr_str(value)?)),
            Some(other) => Err(syn::Error::new_spanned(
                other,
                format!("expected `{key} = \"...\"`"),
            )),
        }
    }

    /// A required `key = "..."`.
    pub fn required_string(&self, key: &str) -> syn::Result<String> {
        self.string(key)?.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("missing required `{key} = \"...\"`"),
            )
        })
    }

    /// `key` present as a bare flag, e.g. `throws_error`.
    pub fn flag(&self, key: &str) -> bool {
        matches!(self.find(key), Some(Meta::Path(_)))
    }

    /// `key = 123`.
    pub fn int(&self, key: &str) -> syn::Result<Option<i64>> {
        match self.find(key) {
            None => Ok(None),
            Some(Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(i), ..
                    }),
                ..
            })) => Ok(Some(i.base10_parse()?)),
            Some(other) => Err(syn::Error::new_spanned(
                other,
                format!("expected `{key} = 123`"),
            )),
        }
    }

    /// `key(en_US = "...", de_DE = "...")`, returning `(locale, content)`
    /// pairs. Underscores in the locale identifier become hyphens
    /// (`en_US` -> `en-US`) since Rust identifiers can't contain them.
    pub fn translations(&self, key: &str) -> syn::Result<Vec<(String, String)>> {
        let Some(Meta::List(list)) = self.find(key) else {
            return Ok(vec![]);
        };
        let entries =
            list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
        entries
            .into_iter()
            .map(|nv| {
                let code = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            &nv.path,
                            "expected a locale identifier, e.g. en_US",
                        )
                    })?
                    .to_string()
                    .replace('_', "-");
                Ok((code, expr_str(&nv.value)?))
            })
            .collect()
    }

    /// `key = ["a", "b"]`.
    pub fn string_array(&self, key: &str) -> syn::Result<Vec<String>> {
        match self.find(key) {
            None => Ok(vec![]),
            Some(Meta::NameValue(MetaNameValue {
                value: Expr::Array(ExprArray { elems, .. }),
                ..
            })) => elems.iter().map(expr_str).collect(),
            Some(other) => Err(syn::Error::new_spanned(
                other,
                format!("expected `{key} = [\"...\", ...]`"),
            )),
        }
    }

    /// `key = [Eager, Lazy]` — a bare-ident array, used for `param_modes`.
    pub fn ident_array(&self, key: &str) -> syn::Result<Vec<syn::Ident>> {
        match self.find(key) {
            None => Ok(vec![]),
            Some(Meta::NameValue(MetaNameValue {
                value: Expr::Array(ExprArray { elems, .. }),
                ..
            })) => {
                elems
                    .iter()
                    .map(|e| match e {
                        Expr::Path(p) => p.path.get_ident().cloned().ok_or_else(|| {
                            syn::Error::new_spanned(p, "expected a bare identifier")
                        }),
                        other => Err(syn::Error::new_spanned(other, "expected a bare identifier")),
                    })
                    .collect()
            }
            Some(other) => Err(syn::Error::new_spanned(
                other,
                format!("expected `{key} = [Eager, Lazy, ...]`"),
            )),
        }
    }
}

/// Extracts and removes every `#[name(...)]` attribute on `attrs`, in source
/// order, parsing each occurrence's arguments as [`AttrArgs`]. This is how
/// `#[parameter(...)]` can appear multiple times on one handler fn: rustc
/// never resolves them as real attributes because the outer
/// `#[taurus_macros::runtime_function]` macro (which runs first) strips them
/// before re-emitting the fn.
pub fn take_repeated(attrs: &mut Vec<Attribute>, name: &str) -> syn::Result<Vec<AttrArgs>> {
    let mut found = Vec::new();
    let mut error = None;
    attrs.retain(|attr: &Attribute| {
        if !attr.path().is_ident(name) {
            return true;
        }
        match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(metas) => found.push(AttrArgs {
                metas: metas.into_iter().collect(),
            }),
            Err(e) => {
                error.get_or_insert(e);
            }
        }
        false
    });
    match error {
        Some(e) => Err(e),
        None => Ok(found),
    }
}

/// `vec![tucana::shared::Translation { code: "..", content: ".." }, ...]`.
pub fn translation_vec(entries: &[(String, String)]) -> TokenStream {
    let items = entries.iter().map(|(code, content)| {
        quote! {
            ::tucana::shared::Translation {
                code: #code.to_string(),
                content: #content.to_string(),
            }
        }
    });
    quote!(vec![#(#items),*])
}

pub fn optional_string(value: Option<String>) -> TokenStream {
    match value {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    }
}
