//! Built-in runtime function catalog.
//!
//! Each submodule registers handler implementations under stable runtime ids.

use crate::handler::registry::FunctionRegistration;

mod array;
mod boolean;
mod control;
mod date;
mod file;
mod http;
mod number;
mod object;
mod text;

// Every function file has now migrated to `#[taurus_macros::runtime_function]`
// self-registration (see `FunctionStore::default`'s `inventory::iter` pass).
// This array is kept (now empty) until the cutover slice removes it, `Self::populate`,
// and `ALL_FUNCTION_SETS` entirely.
pub const ALL_FUNCTION_SETS: &[&[FunctionRegistration]] = &[];
