//! Built-in runtime function catalog.
//!
//! Each submodule registers handler implementations under stable runtime ids.

use crate::handler::registry::FunctionRegistration;

mod array;
mod boolean;
mod control;
mod http;
mod number;
mod object;
mod text;

pub const ALL_FUNCTION_SETS: &[&[FunctionRegistration]] = &[
    array::FUNCTIONS,
    number::FUNCTIONS,
    boolean::FUNCTIONS,
    text::FUNCTIONS,
    object::FUNCTIONS,
    control::FUNCTIONS,
    http::FUNCTIONS,
];
