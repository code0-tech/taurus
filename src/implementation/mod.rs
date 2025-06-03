use crate::registry::HandlerFn;

pub mod boolean;
pub mod number;

pub fn collect() -> Vec<(&'static str, HandlerFn)> {
    let mut result = vec![];

    result.extend(number::collect_number_functions());
    result.extend(boolean::collect_boolean_functions());

    result
}
