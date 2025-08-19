use crate::registry::HandlerFn;

pub mod array;
pub mod boolean;
mod control;
pub mod number;
pub mod object;
pub mod text;

pub fn collect() -> Vec<(&'static str, HandlerFn)> {
    let mut result = vec![];

    result.extend(array::collect_array_functions());
    result.extend(number::collect_number_functions());
    result.extend(boolean::collect_boolean_functions());
    result.extend(text::collect_text_functions());
    result.extend(object::collect_object_functions());
    result.extend(control::collect_control_functions());

    result
}
