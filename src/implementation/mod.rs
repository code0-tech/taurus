use crate::registry::HandlerFn;

mod array;
mod boolean;
mod control;
mod http;
mod number;
mod object;
mod text;

pub fn collect() -> Vec<(&'static str, HandlerFn)> {
    let mut result = vec![];

    result.extend(array::collect_array_functions());
    result.extend(number::collect_number_functions());
    result.extend(boolean::collect_boolean_functions());
    result.extend(text::collect_text_functions());
    result.extend(object::collect_object_functions());
    result.extend(control::collect_control_functions());
    result.extend(http::collect_http_functions());

    result
}
