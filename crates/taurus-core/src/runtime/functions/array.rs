//! List/array runtime handlers.
//!
//! This module includes both pure list transforms and callback-driven handlers
//! (`map`, `filter`, `find`, `for_each`, sort comparators).
//! Callback signals are normalized so `Return(value)` is treated as the
//! callback result value for that iteration.

use std::cmp::Ordering;

use tucana::shared::InputType;
use tucana::shared::{ListValue, Value, value::Kind};

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::{number_to_f64, number_to_string, value_from_f64, value_from_i64};

taurus_macros::module! {
    identifier = "taurus-list",
    name(en_US = "List"),
    description(en_US = "Work with Lists"),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:list",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "LIST",
    module = "taurus-list",
    name(en_US = "Generic List"),
    display_message(en_US = "List of ${T}"),
    alias(en_US = "list;array;collection"),
    generic_keys = ["T"],
    type_string = "T[]",
}

fn as_list(value: &Value, err: &'static str) -> Result<ListValue, RuntimeError> {
    match value.kind.clone().unwrap_or(Kind::NullValue(0)) {
        Kind::ListValue(lv) => Ok(lv),
        _ => Err(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            err,
        )),
    }
}

fn as_bool(value: &Value) -> Result<bool, RuntimeError> {
    match value.kind.clone().unwrap_or(Kind::NullValue(0)) {
        Kind::BoolValue(b) => Ok(b),
        _ => Err(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected boolean result from predicate",
        )),
    }
}

fn fail(category: &str, message: impl Into<String>) -> Signal {
    Signal::Failure(RuntimeError::new("T-STD-00001", category, message))
}

fn parse_array_and_thunk<'a>(
    op_name: &str,
    args: &'a [Argument],
) -> Result<(&'a Value, &'a crate::handler::argument::Thunk), Signal> {
    match args {
        [Argument::Eval(array_v), Argument::Thunk(thunk)] => Ok((array_v, thunk)),
        _ => Err(fail(
            "InvalidArgumentRuntimeError",
            format!(
                "{op_name} expects (array: eager, callback: lazy thunk), got {:?}",
                args
            ),
        )),
    }
}

fn unary_input_type(ctx: &mut ValueStore) -> InputType {
    let node_id = ctx.get_current_node_id();
    InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    }
}

fn run_with_unary_input(
    ctx: &mut ValueStore,
    input_type: InputType,
    iter_index: usize,
    item: &Value,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
    thunk_node: &crate::handler::argument::Thunk,
) -> Signal {
    ctx.insert_input_type(input_type, item.clone());
    ctx.push_runtime_trace_label(|| format!("iter={} value={}", iter_index, preview_value(item)));
    let signal = run(thunk_node, ctx);
    ctx.clear_input_type(input_type);
    signal
}

fn run_with_binary_inputs(
    ctx: &mut ValueStore,
    left_input: InputType,
    right_input: InputType,
    cmp_index: usize,
    left: &Value,
    right: &Value,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
    thunk_node: &crate::handler::argument::Thunk,
) -> Signal {
    ctx.insert_input_type(left_input, left.clone());
    ctx.insert_input_type(right_input, right.clone());
    ctx.push_runtime_trace_label(|| {
        format!(
            "cmp#{} a={} b={}",
            cmp_index,
            preview_value(left),
            preview_value(right)
        )
    });
    let signal = run(thunk_node, ctx);
    ctx.clear_input_type(left_input);
    ctx.clear_input_type(right_input);
    signal
}

fn callback_result_value(signal: Signal) -> Result<Value, Signal> {
    match signal {
        Signal::Success(value) | Signal::Return(value) => Ok(value),
        other @ (Signal::Failure(_) | Signal::Stop) => Err(other),
    }
}

fn comparator_ordering(signal: Signal, reverse: bool) -> Result<Ordering, Signal> {
    let value = callback_result_value(signal)?;

    let ord = match value {
        Value {
            kind: Some(Kind::NumberValue(number)),
        } => match number_to_f64(&number) {
            Some(value) if value < 0.0 => Ordering::Less,
            Some(value) if value > 0.0 => Ordering::Greater,
            Some(_) => Ordering::Equal,
            None => {
                return Err(fail(
                    "InvalidArgumentRuntimeError",
                    "Comparator must return a finite number",
                ));
            }
        },
        value => {
            return Err(fail(
                "InvalidArgumentRuntimeError",
                format!(
                    "Expected comparator to return NumberValue, received {:?}",
                    value
                ),
            ));
        }
    };

    if reverse { Ok(ord.reverse()) } else { Ok(ord) }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::at",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, index: NUMBER): T",
    name(en_US = "Get Element of List"),
    description(en_US = "Retrieves the element at a specified index from a list."),
    display_message(en_US = "Get element at ${index} of ${list}"),
    alias(en_US = "at;array;list;collection;std;index"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
    throws_error,
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list from which to retrieve an element.")
)]
#[parameter(
    runtime_name = "index",
    name(en_US = "Index"),
    description(en_US = "The zero-based index of the element to retrieve.")
)]
fn at(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    // array, index
    args!(args => array: ListValue, index: f64);

    if index < 0.0 {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            "Negative index",
        ));
    }
    let i = index as usize;
    match array.values.get(i) {
        Some(item) => Signal::Success(item.clone()),
        None => fail(
            "IndexOutOfBoundsRuntimeError",
            format!("Index {} out of bounds (len={})", i, array.values.len()),
        ),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::concat",
    module = "taurus-list",
    signature = "<T>(first: LIST<T>, second: LIST<T>): LIST<T>",
    name(en_US = "Combine Lists"),
    description(en_US = "Concatenates/combine two lists into a single list."),
    display_message(en_US = "Combine ${first} with ${second}"),
    alias(en_US = "concat;combine;join;append;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
    throws_error,
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First List"),
    description(en_US = "The first list to concatenate.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second List"),
    description(en_US = "The second list to concatenate.")
)]
fn concat(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => lhs_v: Value, rhs_v: Value);

    let Kind::ListValue(lhs) = lhs_v.kind.clone().ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return fail(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two arrays as arguments but received lhs={:?}",
                lhs_v
            ),
        );
    };
    let Kind::ListValue(rhs) = rhs_v.kind.clone().ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return fail(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two arrays as arguments but received rhs={:?}",
                rhs_v
            ),
        );
    };

    let mut result = lhs.values.clone();
    result.extend(rhs.values.iter().cloned());

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: result })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::filter",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, predicate: PREDICATE<T>): LIST<T>",
    name(en_US = "Filter List"),
    description(en_US = "Returns a new list containing only the elements from the input list for which the predicate returns true."),
    display_message(en_US = "Filter elements in ${list} matching ${predicate}"),
    alias(en_US = "filter;array;list;collection;std"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list to be filtered.")
)]
#[parameter(
    runtime_name = "predicate",
    name(en_US = "Filter Predicate"),
    description(
        en_US = "A function that takes an element of the list and returns a boolean indicating whether the element should be included in the output list."
    )
)]
fn filter(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, predicate_node) = match parse_array_and_thunk("filter", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<Value> = Vec::new();
    let input_type = unary_input_type(ctx);
    for (idx, item) in array.values.iter().enumerate() {
        let pred_sig = run_with_unary_input(ctx, input_type, idx, item, run, predicate_node);
        let predicate_value = match callback_result_value(pred_sig) {
            Ok(v) => v,
            Err(other) => return other,
        };

        match as_bool(&predicate_value) {
            Ok(true) => out.push(item.clone()),
            Ok(false) => {}
            Err(e) => return Signal::Failure(e),
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: out })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::find",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, predicate: PREDICATE<T>): T",
    name(en_US = "Find Element in List"),
    description(en_US = "Returns the first element from the input list for which the predicate returns true. If no element matches, returns null."),
    display_message(en_US = "Find first element in ${list} matching ${predicate}"),
    alias(en_US = "find;array;list;collection;std"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST", "PREDICATE"],
    throws_error,
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list in which an element satisfying the predicate will be searched.")
)]
#[parameter(
    runtime_name = "predicate",
    name(en_US = "Search Predicate"),
    description(
        en_US = "A function that takes an element of the list and returns a boolean indicating if the element matches the search criteria."
    )
)]
fn find(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, predicate_node) = match parse_array_and_thunk("find", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let input_type = unary_input_type(ctx);

    for (idx, item) in array.values.iter().enumerate() {
        let pred_sig = run_with_unary_input(ctx, input_type, idx, item, run, predicate_node);
        let predicate_value = match callback_result_value(pred_sig) {
            Ok(v) => v,
            Err(other) => return other,
        };
        match as_bool(&predicate_value) {
            Ok(true) => return Signal::Success(item.clone()),
            Ok(false) => continue,
            Err(e) => return Signal::Failure(e),
        }
    }

    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}
#[taurus_macros::runtime_function(
    identifier = "std::list::find_last",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, predicate: PREDICATE<T>): T",
    name(en_US = "Find Last Element in List"),
    description(en_US = "Returns the last element from the input list for which the predicate returns true. If no element matches, returns null or equivalent."),
    display_message(en_US = "Last Element of ${list} matching ${predicate}"),
    alias(en_US = "find last;last index;last position;array;list;collection;std;find;last"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST", "PREDICATE"],
    throws_error,
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list in which an element satisfying the predicate will be searched.")
)]
#[parameter(
    runtime_name = "predicate",
    name(en_US = "Search Predicate"),
    description(
        en_US = "A function that takes an element of the list and returns a boolean indicating if the element matches the search criteria."
    )
)]
fn find_last(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, predicate_node) = match parse_array_and_thunk("find_last", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let input_type = unary_input_type(ctx);

    for (idx, item) in array.values.iter().enumerate().rev() {
        let pred_sig = run_with_unary_input(ctx, input_type, idx, item, run, predicate_node);
        let predicate_value = match callback_result_value(pred_sig) {
            Ok(v) => v,
            Err(other) => return other,
        };
        match as_bool(&predicate_value) {
            Ok(true) => return Signal::Success(item.clone()),
            Ok(false) => continue,
            Err(e) => return Signal::Failure(e),
        }
    }

    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}

#[taurus_macros::runtime_function(
    identifier = "std::list::find_index",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, predicate: PREDICATE<T>): NUMBER",
    name(en_US = "Find Index of Element in List"),
    description(en_US = "Returns the zero-based index of the first element for which the predicate returns true. If no element matches, returns -1."),
    display_message(en_US = "Index of Element in ${list} matching ${predicate}"),
    alias(en_US = "find index;index of;position;array;list;collection;std;find;index"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST", "NUMBER", "PREDICATE"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(
        en_US = "The list in which to find the index of an element that satisfies the predicate."
    )
)]
#[parameter(
    runtime_name = "predicate",
    name(en_US = "Search Predicate"),
    description(
        en_US = "A function that takes an element of the list and returns a boolean indicating if the element satisfies the search criteria."
    )
)]
fn find_index(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, predicate_node) = match parse_array_and_thunk("find_index", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let input_type = unary_input_type(ctx);

    for (idx, item) in array.values.iter().enumerate() {
        let pred_sig = run_with_unary_input(ctx, input_type, idx, item, run, predicate_node);
        let predicate_value = match callback_result_value(pred_sig) {
            Ok(v) => v,
            Err(other) => return other,
        };

        match as_bool(&predicate_value) {
            Ok(true) => return Signal::Success(value_from_i64(idx as i64)),
            Ok(false) => continue,
            Err(e) => return Signal::Failure(e),
        }
    }

    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}
#[taurus_macros::runtime_function(
    identifier = "std::list::first",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): T",
    name(en_US = "First Element of List"),
    description(en_US = "Retrieves the first element from the list."),
    display_message(en_US = "Get First Element in ${list}"),
    alias(en_US = "first;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list from which to retrieve the first element.")
)]
fn first(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue);

    match array.values.first() {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::last",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): T",
    name(en_US = "Last Element of List"),
    description(en_US = "Retrieves the last element from the list."),
    display_message(en_US = "Get Last Element of ${list}"),
    alias(en_US = "last;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(en_US = "The list from which to retrieve the last element.")
)]
fn last(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue);
    match array.values.last() {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::for_each",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, consumer: CONSUMER<T>): void",
    name(en_US = "For Each Element"),
    description(en_US = "Executes a consumer function for each element in the list."),
    display_message(en_US = "For each in ${list} do ${consumer}"),
    alias(en_US = "for_each;array;list;collection;std;for;each"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST", "CONSUMER"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(
        en_US = "Each element of this list will be passed to the provided consumer function for processing."
    )
)]
#[parameter(
    runtime_name = "consumer",
    name(en_US = "Consumer Function"),
    description(
        en_US = "This function is invoked once for each element in the list. It is not expected to return a value."
    )
)]
fn for_each(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, transform_node) = match parse_array_and_thunk("for_each", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let input_type = unary_input_type(ctx);

    for (idx, item) in array.values.iter().enumerate() {
        let sig = run_with_unary_input(ctx, input_type, idx, item, run, transform_node);

        match callback_result_value(sig) {
            Ok(_) => {}
            Err(other) => return other,
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::NullValue(0)),
    })
}

fn preview_value(value: &Value) -> String {
    format_value_json(value)
}

fn format_value_json(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::NumberValue(v)) => number_to_string(v),
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::StringValue(v)) => format!("{:?}", v),
        Some(Kind::NullValue(_)) | None => "null".to_string(),
        Some(Kind::ListValue(list)) => {
            let mut parts = Vec::new();
            for item in list.values.iter() {
                parts.push(format_value_json(item));
            }
            format!("[{}]", parts.join(", "))
        }
        Some(Kind::StructValue(struct_value)) => {
            let mut keys: Vec<_> = struct_value.fields.keys().collect();
            keys.sort();
            let mut parts = Vec::new();
            for key in keys.iter() {
                if let Some(v) = struct_value.fields.get(*key) {
                    parts.push(format!("{:?}: {}", key, format_value_json(v)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::map",
    module = "taurus-list",
    signature = "<T, R>(list: LIST<T>, transform: TRANSFORM<T, R>): LIST<R>",
    name(en_US = "Map List"),
    description(en_US = "Transforms each element in the list using the provided function. This will create a new list of new elements which will be returned."),
    display_message(en_US = "Apply ${transform} for each in ${list}"),
    alias(en_US = "map;array;list;collection;std"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST", "TRANSFORM"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Input List"),
    description(
        en_US = "Each element of this list will be passed through the transform function."
    )
)]
#[parameter(
    runtime_name = "transform",
    name(en_US = "Transform Function"),
    description(
        en_US = "The transform function is applied to every element of the list to produce a new list."
    )
)]
fn map(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, transform_node) = match parse_array_and_thunk("map", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<Value> = Vec::with_capacity(array.values.len());
    let input_type = unary_input_type(ctx);

    for (idx, item) in array.values.iter().enumerate() {
        let sig = run_with_unary_input(ctx, input_type, idx, item, run, transform_node);
        match callback_result_value(sig) {
            Ok(v) => out.push(v),
            Err(other) => return other,
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: out })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::push",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, item: T): NUMBER",
    name(en_US = "Push to List"),
    description(en_US = "Adds a new element to the end of the list and returns the new length of the list."),
    display_message(en_US = "Push ${item} into ${list}"),
    alias(en_US = "push;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The list to which an item will be added.")
)]
#[parameter(
    runtime_name = "item",
    name(en_US = "Item"),
    description(en_US = "The value to be added at the end of the list.")
)]
fn push(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected first argument to be an array",
        ));
    };
    array.values.push(item);
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::pop",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): T",
    name(en_US = "Pop from List"),
    description(en_US = "Removes the last element from the specified list and returns it. The list will be modified."),
    display_message(en_US = "Remove Last Item of ${list}"),
    alias(en_US = "pop;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The list to remove the last item from.")
)]
fn pop(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    array.values.pop();
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::remove",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, item: T): LIST<T>",
    name(en_US = "Remove from List"),
    description(en_US = "Removes the first matching item from the given list and returns the resulting list."),
    display_message(en_US = "Remove ${item} from ${list}"),
    alias(en_US = "remove;delete;strip;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The list from which the item will be removed.")
)]
#[parameter(
    runtime_name = "item",
    name(en_US = "Item"),
    description(en_US = "The item to remove from the list.")
)]
fn remove(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected first argument to be an array",
        ));
    };

    if let Some(index) = array.values.iter().position(|x| *x == item) {
        array.values.remove(index);
        Signal::Success(Value {
            kind: Some(Kind::ListValue(array)),
        })
    } else {
        fail(
            "ValueNotFoundRuntimeError",
            format!("Item {:?} not found in array", item),
        )
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::is_empty",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): BOOLEAN",
    name(en_US = "Is List Empty"),
    description(en_US = "Returns true if the list contains no elements, otherwise returns false."),
    display_message(en_US = "Check if ${list} is empty"),
    alias(en_US = "is_empty;array;list;collection;std;is;empty"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The list to check for emptiness.")
)]
fn is_empty(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(array.values.is_empty())),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::size",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): NUMBER",
    name(en_US = "List Size"),
    description(en_US = "This function returns the count of elements present in the given list."),
    display_message(en_US = "Size of ${list}"),
    alias(en_US = "size;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["NUMBER", "LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The list whose number of elements is to be returned.")
)]
fn size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    Signal::Success(value_from_i64(array.values.len() as i64))
}

#[taurus_macros::runtime_function(
    identifier = "std::list::index_of",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, item: T): NUMBER",
    name(en_US = "Index of Item"),
    description(en_US = "Returns the zero-based index of the first occurrence of a given item in the specified list. If the item is not found, it typically returns -1."),
    display_message(en_US = "Get Index of ${item} in ${list}"),
    alias(en_US = "index_of;array;list;collection;std;index;of"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(
        en_US = "A list of elements in which the specified item will be searched for to determine its index."
    )
)]
#[parameter(
    runtime_name = "item",
    name(en_US = "Item"),
    description(
        en_US = "The item for which the function searches in the list and returns the index of its first occurrence."
    )
)]
fn index_of(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected first argument to be an array",
        ));
    };

    match array.values.iter().position(|x| *x == item) {
        Some(i) => Signal::Success(value_from_i64(i as i64)),
        None => fail(
            "ValueNotFoundRuntimeError",
            format!("Item {:?} not found in array", item),
        ),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::to_unique",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): LIST<T>",
    name(en_US = "To Unique"),
    description(en_US = "Returns a new list containing only the unique elements from the input list."),
    display_message(en_US = "Remove duplicates in ${list}"),
    alias(en_US = "to_unique;array;list;collection;std;to;unique"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The input list from which duplicates will be removed.")
)]
fn to_unique(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };

    let mut unique = Vec::<Value>::new();
    for v in &array.values {
        if !unique.contains(v) {
            unique.push(v.clone());
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: unique })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::sort",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, comparator: COMPARATOR<T>): LIST<T>",
    name(en_US = "Sort List"),
    description(en_US = "Returns a new list with the elements sorted according to the comparator function provided."),
    display_message(en_US = "Sort ${list} using ${comparator}"),
    alias(en_US = "sort;array;list;collection;std"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The input list to be sorted.")
)]
#[parameter(
    runtime_name = "comparator",
    name(en_US = "Comparator"),
    description(
        en_US = "A function that takes two elements and returns a negative, zero, or positive number to indicate their ordering."
    )
)]
fn sort(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, transform_node) = match parse_array_and_thunk("sort", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let mut array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let node_id = ctx.get_current_node_id();
    let left_input = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    let right_input = InputType {
        node_id,
        parameter_index: 1,
        input_index: 1,
    };

    let mut comparator_failure: Option<Signal> = None;
    let mut cmp_idx = 0usize;
    array.values.sort_by(|left, right| {
        if comparator_failure.is_some() {
            return Ordering::Equal;
        }

        let signal = run_with_binary_inputs(
            ctx,
            left_input,
            right_input,
            cmp_idx,
            left,
            right,
            run,
            transform_node,
        );
        cmp_idx += 1;

        match comparator_ordering(signal, false) {
            Ok(ordering) => ordering,
            Err(signal) => {
                comparator_failure = Some(signal);
                Ordering::Equal
            }
        }
    });

    if let Some(signal) = comparator_failure {
        return signal;
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::sort_reverse",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>, comparator: COMPARATOR<T>): LIST<T>",
    name(en_US = "Sort List in Reverse"),
    description(en_US = "Returns a new list with the elements sorted in descending order according to the comparator function provided."),
    display_message(en_US = "Reversed-Sort ${list} using ${comparator}"),
    alias(en_US = "sort_reverse;array;list;collection;std;sort;reverse"),
    display_icon = "tabler:arrow-iteration",
    linked_data_type_identifiers = ["LIST"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The input list to be sorted in reverse order.")
)]
#[parameter(
    runtime_name = "comparator",
    name(en_US = "Comparator"),
    description(
        en_US = "A function that takes two elements and returns a negative, zero, or positive number to indicate their ordering."
    )
)]
fn sort_reverse(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let (array_v, transform_node) = match parse_array_and_thunk("sort_reverse", args) {
        Ok(data) => data,
        Err(signal) => return signal,
    };

    let mut array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let node_id = ctx.get_current_node_id();
    let left_input = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    let right_input = InputType {
        node_id,
        parameter_index: 1,
        input_index: 1,
    };

    let mut comparator_failure: Option<Signal> = None;
    let mut cmp_idx = 0usize;
    array.values.sort_by(|left, right| {
        if comparator_failure.is_some() {
            return Ordering::Equal;
        }

        let signal = run_with_binary_inputs(
            ctx,
            left_input,
            right_input,
            cmp_idx,
            left,
            right,
            run,
            transform_node,
        );
        cmp_idx += 1;

        match comparator_ordering(signal, true) {
            Ok(ordering) => ordering,
            Err(signal) => {
                comparator_failure = Some(signal);
                Ordering::Equal
            }
        }
    });

    if let Some(signal) = comparator_failure {
        return signal;
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::reverse",
    module = "taurus-list",
    signature = "<T>(list: LIST<T>): LIST<T>",
    name(en_US = "Reverse List"),
    description(en_US = "Returns a new list with the elements of the input list in reverse order."),
    display_message(en_US = "Reverse ${list}"),
    alias(en_US = "reverse;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List"),
    description(en_US = "The input list to be reversed.")
)]
fn reverse(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    array.values.reverse();
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::flat",
    module = "taurus-list",
    signature = "<T>(list: LIST<LIST<T>>): LIST<T>",
    name(en_US = "Flatten List"),
    description(en_US = "Flattens a nested list into a single-level list."),
    display_message(en_US = "Flatten ${list}"),
    alias(en_US = "flat;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Nested List"),
    description(
        en_US = "A list containing sub-lists that will be flattened into a single-level list."
    )
)]
fn flat(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };

    let mut out: Vec<Value> = Vec::new();
    for item in &array.values {
        match &item.kind {
            Some(Kind::ListValue(sub)) => out.extend(sub.values.iter().cloned()),
            _ => out.push(item.clone()),
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: out })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::list::min",
    module = "taurus-list",
    signature = "(list: LIST<NUMBER>): NUMBER",
    name(en_US = "Find Minimum Number"),
    description(en_US = "Finds the minimum value in a numeric list."),
    display_message(en_US = "Minimum of ${list}"),
    alias(en_US = "min;minimum;smallest;least;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Number List"),
    description(en_US = "A list of numbers to find the minimum value from.")
)]
fn min(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue);

    let mut nums: Vec<f64> = Vec::new();
    let mut all_int = true;
    let mut min_i64: Option<i64> = None;
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = &v.kind {
            match n.number {
                Some(tucana::shared::number_value::Number::Integer(i)) => {
                    min_i64 = Some(match min_i64 {
                        Some(curr) => curr.min(i),
                        None => i,
                    });
                    nums.push(i as f64);
                }
                Some(tucana::shared::number_value::Number::Float(f)) => {
                    all_int = false;
                    nums.push(f);
                }
                None => {}
            }
        }
    }

    match nums.iter().min_by(|a, b| a.total_cmp(b)) {
        Some(m) if all_int => Signal::Success(value_from_i64(min_i64.unwrap_or(*m as i64))),
        Some(m) => Signal::Success(value_from_f64(*m)),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "ArrayEmptyRuntimeError",
            "Array is empty",
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::max",
    module = "taurus-list",
    signature = "(list: LIST<NUMBER>): NUMBER",
    name(en_US = "Find Maximum Number"),
    description(en_US = "Finds the maximum value in a numeric list."),
    display_message(en_US = "Maximum of ${list}"),
    alias(en_US = "max;maximum;largest;greatest;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List of Numbers"),
    description(en_US = "A list of numbers to find the maximum value from.")
)]
fn max(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue);

    let mut nums: Vec<f64> = Vec::new();
    let mut all_int = true;
    let mut max_i64: Option<i64> = None;
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = &v.kind {
            match n.number {
                Some(tucana::shared::number_value::Number::Integer(i)) => {
                    max_i64 = Some(match max_i64 {
                        Some(curr) => curr.max(i),
                        None => i,
                    });
                    nums.push(i as f64);
                }
                Some(tucana::shared::number_value::Number::Float(f)) => {
                    all_int = false;
                    nums.push(f);
                }
                None => {}
            }
        }
    }

    match nums.iter().max_by(|a, b| a.total_cmp(b)) {
        Some(m) if all_int => Signal::Success(value_from_i64(max_i64.unwrap_or(*m as i64))),
        Some(m) => Signal::Success(value_from_f64(*m)),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "ArrayEmptyRuntimeError",
            "Array is empty",
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::sum",
    module = "taurus-list",
    signature = "(list: LIST<NUMBER>): NUMBER",
    name(en_US = "Sum of Numbers"),
    description(en_US = "Returns the total sum of the elements in the numeric list."),
    display_message(en_US = "Sum of ${list}"),
    alias(en_US = "sum;total;add all;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "Number List"),
    description(en_US = "Calculates the sum of all numbers in the given list.")
)]
fn sum(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue);

    let mut s_f = 0.0;
    let mut s_i: i64 = 0;
    let mut all_int = true;
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = &v.kind {
            match n.number {
                Some(tucana::shared::number_value::Number::Integer(i)) => {
                    if let Some(next) = s_i.checked_add(i) {
                        s_i = next;
                        s_f += i as f64;
                    } else {
                        all_int = false;
                        if let Some(f) = number_to_f64(n) {
                            s_f += f;
                        }
                    }
                }
                Some(tucana::shared::number_value::Number::Float(f)) => {
                    all_int = false;
                    s_f += f;
                }
                None => {}
            }
        }
    }

    if all_int {
        Signal::Success(value_from_i64(s_i))
    } else {
        Signal::Success(value_from_f64(s_f))
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::list::join",
    module = "taurus-list",
    signature = "(list: LIST<TEXT>, join_text: TEXT): TEXT",
    name(en_US = "Join Text List"),
    description(en_US = "Returns a single concatenated string of text joined by the provided join text."),
    display_message(en_US = "Joins ${list} using '${join_text}'"),
    alias(en_US = "join;array;list;collection;std"),
    display_icon = "tabler:list",
    linked_data_type_identifiers = ["LIST", "TEXT"],
)]
#[parameter(
    runtime_name = "list",
    name(en_US = "List of Text"),
    description(en_US = "A list of text elements to combined into a single word.")
)]
#[parameter(
    runtime_name = "join_text",
    name(en_US = "Join Text"),
    description(
        en_US = "The text that will be used to join the list elements into a single string."
    )
)]
fn join(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => array: ListValue, separator: String);

    let mut parts: Vec<String> = Vec::new();
    for v in &array.values {
        if let Some(Kind::StringValue(s)) = &v.kind {
            parts.push(s.clone());
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::StringValue(parts.join(&separator))),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, number_value_from_f64, value_from_f64};
    use tucana::shared::{ListValue, Value, value::Kind};

    // --- helpers -------------------------------------------------------------
    fn a_val(v: Value) -> Argument {
        Argument::Eval(v)
    }
    fn a_thunk(id: i64) -> Argument {
        Argument::Thunk(crate::handler::argument::Thunk::Node(id))
    }
    fn v_num(n: f64) -> Value {
        value_from_f64(n)
    }
    fn v_str(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }
    fn v_bool(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }
    fn v_list(values: Vec<Value>) -> Value {
        Value {
            kind: Some(Kind::ListValue(ListValue { values })),
        }
    }
    fn k_num(n: f64) -> Option<Kind> {
        Some(Kind::NumberValue(number_value_from_f64(n)))
    }

    fn expect_num(sig: Signal) -> f64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_f64(&n).unwrap_or_default(),
            x => panic!("Expected NumberValue, got {:?}", x),
        }
    }
    fn expect_str(sig: Signal) -> String {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::StringValue(s)),
            }) => s,
            x => panic!("Expected StringValue, got {:?}", x),
        }
    }
    fn expect_list(sig: Signal) -> Vec<Value> {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::ListValue(ListValue { values })),
            }) => values,
            x => panic!("Expected ListValue, got {:?}", x),
        }
    }
    fn expect_bool(sig: Signal) -> bool {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::BoolValue(b)),
            }) => b,
            x => panic!("Expected BoolValue, got {:?}", x),
        }
    }

    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    fn run_from_bools(
        seq: Vec<bool>,
    ) -> impl FnMut(&crate::handler::argument::Thunk, &mut ValueStore) -> Signal {
        let mut i = 0usize;
        move |_, _| {
            let b = *seq.get(i).unwrap_or(&false);
            i += 1;
            Signal::Success(Value {
                kind: Some(Kind::BoolValue(b)),
            })
        }
    }

    fn run_from_values(
        seq: Vec<Value>,
    ) -> impl FnMut(&crate::handler::argument::Thunk, &mut ValueStore) -> Signal {
        let mut i = 0usize;
        move |_, _| {
            let v = seq.get(i).cloned().unwrap_or(Value {
                kind: Some(Kind::NullValue(0)),
            });
            i += 1;
            Signal::Success(v)
        }
    }

    // --- at ------------------------------------------------------------------
    #[test]
    fn test_at_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_num(10.0), v_num(20.0), v_num(30.0)]);

        assert_eq!(
            expect_num(at(
                &[a_val(arr.clone()), a_val(v_num(0.0))],
                &mut ctx,
                &mut run
            )),
            10.0
        );
        assert_eq!(
            expect_num(at(
                &[a_val(arr.clone()), a_val(v_num(1.0))],
                &mut ctx,
                &mut run
            )),
            20.0
        );
        assert_eq!(
            expect_num(at(&[a_val(arr), a_val(v_num(2.0))], &mut ctx, &mut run)),
            30.0
        );
    }

    #[test]
    fn test_at_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_num(1.0)]);

        // wrong arg count
        match at(&[a_val(arr.clone())], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        // wrong type first arg
        match at(
            &[a_val(v_str("not_array")), a_val(v_num(0.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        // wrong type second arg
        match at(&[a_val(arr.clone()), a_val(v_str("x"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        // oob / negative
        match at(&[a_val(arr.clone()), a_val(v_num(9.0))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match at(&[a_val(arr), a_val(v_num(-1.0))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- concat --------------------------------------------------------------
    #[test]
    fn test_concat_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let a = v_list(vec![v_num(1.0), v_num(2.0)]);
        let b = v_list(vec![v_num(3.0), v_num(4.0)]);
        let out = expect_list(concat(&[a_val(a), a_val(b)], &mut ctx, &mut run));
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].kind, k_num(1.0));
        assert_eq!(out[3].kind, k_num(4.0));
    }

    #[test]
    fn test_concat_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_num(1.0)]);
        match concat(&[a_val(arr.clone())], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match concat(
            &[a_val(v_str("not_array")), a_val(arr.clone())],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match concat(&[a_val(arr), a_val(v_num(42.0))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- filter / find / find_last / find_index ------------------------------
    #[test]
    fn test_filter_success() {
        let mut ctx = ValueStore::default();
        let array = v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]);
        let mut run = run_from_bools(vec![true, false, true]);
        let out = expect_list(filter(&[a_val(array), a_thunk(1)], &mut ctx, &mut run));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, k_num(1.0));
        assert_eq!(out[1].kind, k_num(3.0));
    }

    #[test]
    fn test_filter_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let array = v_list(vec![v_num(1.0)]);
        let _predicate = v_list(vec![v_bool(true)]);
        match filter(&[a_val(array.clone())], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match filter(&[a_val(v_str("not_array")), a_thunk(1)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match filter(&[a_val(array), a_val(v_num(1.0))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- first / last --------------------------------------------------------
    #[test]
    fn test_first_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_str("first"), v_str("second"), v_str("third")]);
        assert_eq!(
            expect_str(first(&[a_val(arr)], &mut ctx, &mut run)),
            "first"
        );
    }

    #[test]
    fn test_first_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match first(&[a_val(v_list(vec![]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match first(&[a_val(v_str("not_array"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match first(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_last_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_str("first"), v_str("second"), v_str("last")]);
        assert_eq!(expect_str(last(&[a_val(arr)], &mut ctx, &mut run)), "last");
    }

    #[test]
    fn test_last_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match last(&[a_val(v_list(vec![]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match last(&[a_val(v_str("not_array"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- for_each / map ------------------------------------------------------
    #[test]
    fn test_for_each_and_map() {
        let mut ctx = ValueStore::default();
        let mut called = 0usize;
        let mut run = |_: &crate::handler::argument::Thunk, _ctx: &mut ValueStore| {
            called += 1;
            Signal::Success(Value {
                kind: Some(Kind::NullValue(0)),
            })
        };
        match for_each(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0)])), a_thunk(1)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Success(Value {
                kind: Some(Kind::NullValue(_)),
            }) => {}
            x => panic!("expected NullValue, got {:?}", x),
        }
        assert_eq!(called, 2);
        let transformed = v_list(vec![v_str("X"), v_str("Y")]);
        let mut run = run_from_values(match transformed.kind.clone() {
            Some(Kind::ListValue(ListValue { values })) => values,
            _ => unreachable!(),
        });
        let out = expect_list(map(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0)])), a_thunk(2)],
            &mut ctx,
            &mut run,
        ));
        let expected = match transformed.kind {
            Some(Kind::ListValue(ListValue { values })) => values,
            _ => unreachable!(),
        };
        assert_eq!(out, expected);
    }

    #[test]
    fn test_for_each_and_map_treat_callback_return_as_local_result() {
        let mut ctx = ValueStore::default();
        let return_value = v_str("early_return");

        let mut for_each_calls = 0usize;
        let mut for_each_run = |_: &crate::handler::argument::Thunk, _ctx: &mut ValueStore| {
            for_each_calls += 1;
            Signal::Return(return_value.clone())
        };
        let for_each_result = for_each(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0)])), a_thunk(1)],
            &mut ctx,
            &mut for_each_run,
        );
        match for_each_result {
            Signal::Success(Value {
                kind: Some(Kind::NullValue(_)),
            }) => {}
            other => panic!("expected Success(NullValue), got {:?}", other),
        }
        assert_eq!(for_each_calls, 2);

        let mut map_calls = 0usize;
        let mut map_run = |_: &crate::handler::argument::Thunk, _ctx: &mut ValueStore| {
            map_calls += 1;
            Signal::Return(return_value.clone())
        };
        let map_result = map(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0)])), a_thunk(2)],
            &mut ctx,
            &mut map_run,
        );
        match map_result {
            Signal::Success(Value {
                kind: Some(Kind::ListValue(ListValue { values })),
            }) => {
                assert_eq!(values, vec![return_value.clone(), return_value.clone()]);
            }
            other => panic!(
                "expected Success([return_value, return_value]), got {:?}",
                other
            ),
        }
        assert_eq!(map_calls, 2);
    }

    #[test]
    fn test_filter_and_find_use_return_as_callback_value() {
        let mut ctx = ValueStore::default();

        let mut filter_index = 0usize;
        let filter_returns = [true, false, true];
        let mut filter_run = |_: &crate::handler::argument::Thunk, _ctx: &mut ValueStore| {
            let out = filter_returns.get(filter_index).copied().unwrap_or(false);
            filter_index += 1;
            Signal::Return(v_bool(out))
        };
        let filtered = expect_list(filter(
            &[
                a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)])),
                a_thunk(1),
            ],
            &mut ctx,
            &mut filter_run,
        ));
        assert_eq!(filtered, vec![v_num(1.0), v_num(3.0)]);

        let mut find_index = 0usize;
        let find_returns = [false, true];
        let mut find_run = |_: &crate::handler::argument::Thunk, _ctx: &mut ValueStore| {
            let out = find_returns.get(find_index).copied().unwrap_or(false);
            find_index += 1;
            Signal::Return(v_bool(out))
        };
        let found = find(
            &[a_val(v_list(vec![v_str("A"), v_str("B")])), a_thunk(2)],
            &mut ctx,
            &mut find_run,
        );
        assert_eq!(expect_str(found), "B");
    }

    // --- push / pop / remove -------------------------------------------------
    #[test]
    fn test_push_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let out = expect_list(push(
            &[
                a_val(v_list(vec![v_num(1.0), v_num(2.0)])),
                a_val(v_num(3.0)),
            ],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 3);
        assert_eq!(out[2].kind, k_num(3.0));
    }

    #[test]
    fn test_push_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match push(&[a_val(v_list(vec![v_num(1.0)]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match push(
            &[a_val(v_str("nope")), a_val(v_num(1.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_pop_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let out = expect_list(pop(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]))],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, k_num(1.0));
        assert_eq!(out[1].kind, k_num(2.0));
    }

    #[test]
    fn test_pop_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match pop(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match pop(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_remove_success_and_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        // success
        let arr = v_list(vec![v_str("first"), v_str("second"), v_str("third")]);
        let out = expect_list(remove(
            &[a_val(arr), a_val(v_str("second"))],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, Some(Kind::StringValue("first".into())));
        assert_eq!(out[1].kind, Some(Kind::StringValue("third".into())));
        // errors
        match remove(&[a_val(v_list(vec![v_num(1.0)]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match remove(
            &[a_val(v_str("nope")), a_val(v_num(0.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match remove(
            &[a_val(v_list(vec![v_num(1.0)])), a_val(v_num(999.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- is_empty / size -----------------------------------------------------
    #[test]
    fn test_is_empty_and_size() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        assert!(expect_bool(is_empty(
            &[a_val(v_list(vec![]))],
            &mut ctx,
            &mut run
        )));
        assert!(!expect_bool(is_empty(
            &[a_val(v_list(vec![v_num(1.0)]))],
            &mut ctx,
            &mut run
        )));
        assert_eq!(
            expect_num(size(&[a_val(v_list(vec![]))], &mut ctx, &mut run)),
            0.0
        );
        assert_eq!(
            expect_num(size(
                &[a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]))],
                &mut ctx,
                &mut run
            )),
            3.0
        );
    }

    #[test]
    fn test_is_empty_error_and_size_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match is_empty(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match is_empty(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match size(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match size(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- index_of / to_unique ------------------------------------------------
    #[test]
    fn test_index_of_and_to_unique() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_num(10.0), v_num(42.0), v_num(30.0), v_num(42.0)]);
        assert_eq!(
            expect_num(index_of(
                &[a_val(arr.clone()), a_val(v_num(42.0))],
                &mut ctx,
                &mut run
            )),
            1.0
        );
        match index_of(
            &[a_val(arr.clone()), a_val(v_num(999.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }

        let uniq = expect_list(to_unique(&[a_val(arr)], &mut ctx, &mut run));
        assert_eq!(uniq.len(), 3);
        assert_eq!(uniq[0].kind, k_num(10.0));
        assert_eq!(uniq[1].kind, k_num(42.0));
        assert_eq!(uniq[2].kind, k_num(30.0));
    }

    #[test]
    fn test_index_of_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match index_of(&[a_val(v_list(vec![v_num(1.0)]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match index_of(
            &[a_val(v_str("nope")), a_val(v_num(1.0))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- sort / sort_reverse -------------------------------------------------
    #[test]
    fn test_sort_and_sort_reverse() {
        let mut ctx = ValueStore::default();

        // We don't rely on actual values; ordering is driven by the comparator sequence.
        let arr = v_list(vec![v_str("a"), v_str("b"), v_str("c"), v_str("d")]);
        let comps = vec![v_num(-1.0), v_num(1.0), v_num(0.0), v_num(-1.0)];
        let mut run = run_from_values(comps.clone());
        let out = expect_list(sort(&[a_val(arr.clone()), a_thunk(1)], &mut ctx, &mut run));
        assert_eq!(out.len(), 4);

        let mut run = run_from_values(comps);
        let out_r = expect_list(sort_reverse(&[a_val(arr), a_thunk(1)], &mut ctx, &mut run));
        assert_eq!(out_r.len(), 4);
    }

    // --- reverse / flat ------------------------------------------------------
    #[test]
    fn test_reverse_success_and_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let out = expect_list(reverse(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]))],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out[0].kind, k_num(3.0));
        assert_eq!(out[2].kind, k_num(1.0));

        match reverse(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match reverse(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    #[test]
    fn test_flat_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let nested = v_list(vec![
            v_num(1.0),
            v_list(vec![v_num(2.0), v_num(3.0)]),
            v_list(vec![]),
            v_num(4.0),
        ]);
        let out = expect_list(flat(&[a_val(nested)], &mut ctx, &mut run));
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].kind, k_num(1.0));
        assert_eq!(out[1].kind, k_num(2.0));
        assert_eq!(out[2].kind, k_num(3.0));
        assert_eq!(out[3].kind, k_num(4.0));
    }

    // --- min / max / sum -----------------------------------------------------
    #[test]
    fn test_min_max_sum_success_and_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let nums = v_list(vec![v_num(5.0), v_num(1.0), v_num(8.0), v_num(2.0)]);
        assert_eq!(
            expect_num(min(&[a_val(nums.clone())], &mut ctx, &mut run)),
            1.0
        );
        assert_eq!(
            expect_num(max(&[a_val(nums.clone())], &mut ctx, &mut run)),
            8.0
        );
        assert_eq!(expect_num(sum(&[a_val(nums)], &mut ctx, &mut run)), 16.0);

        // empty
        match min(&[a_val(v_list(vec![]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match max(&[a_val(v_list(vec![]))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        assert_eq!(
            expect_num(sum(&[a_val(v_list(vec![]))], &mut ctx, &mut run)),
            0.0
        );

        // wrong type
        match sum(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match min(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match max(&[a_val(v_str("nope"))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }

    // --- join ----------------------------------------------------------------
    #[test]
    fn test_join_success_and_error() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_str("hello"), v_str("world"), v_str("test")]);
        assert_eq!(
            expect_str(join(&[a_val(arr), a_val(v_str(", "))], &mut ctx, &mut run)),
            "hello, world, test"
        );

        // errors
        let arr2 = v_list(vec![v_str("hello")]);
        match join(&[a_val(arr2.clone())], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match join(
            &[a_val(v_str("not_array")), a_val(v_str(","))],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match join(&[a_val(arr2), a_val(v_num(42.0))], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
    }
}
