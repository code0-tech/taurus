//! Built-in runtime function catalog.
//!
//! Each submodule registers its handler implementations and metadata via
//! `#[taurus_macros::runtime_function]` self-registration (see
//! `FunctionStore::default`'s `inventory::iter` pass and
//! `taurus_core::registry::build_modules`).

mod array;
mod boolean;
mod color;
mod control;
mod date;
mod file;
mod http;
mod number;
mod object;
mod text;
