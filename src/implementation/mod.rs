use crate::registry::HandlerFn;

pub mod array;
pub mod boolean;
pub mod number;
pub mod text;

pub fn collect() -> Vec<(&'static str, HandlerFn)> {
    let mut result = vec![];

    result.extend(array::collect_array_functions());
    result.extend(number::collect_number_functions());
    result.extend(boolean::collect_boolean_functions());
    result.extend(text::collect_text_functions());

    result
}
