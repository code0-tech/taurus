use std::cmp::Ordering;

use tucana::shared::InputType;
use tucana::shared::{ListValue, Value, value::Kind};

use crate::context::argument::Argument;
use crate::context::argument::ParameterNode::{Eager, Lazy};
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::{context::context::Context, error::RuntimeError};

pub fn collect_array_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("std::list::at", HandlerFn::eager(at, 2)),
        ("std::list::concat", HandlerFn::eager(concat, 2)),
        (
            "std::list::filter",
            HandlerFn::into_function_entry(filter, vec![Eager, Lazy]),
        ),
        (
            "std::list::find",
            HandlerFn::into_function_entry(find, vec![Eager, Lazy]),
        ),
        (
            "std::list::find_last",
            HandlerFn::into_function_entry(find_last, vec![Eager, Lazy]),
        ),
        (
            "std::list::find_index",
            HandlerFn::into_function_entry(find_index, vec![Eager, Lazy]),
        ),
        ("std::list::first", HandlerFn::eager(first, 1)),
        ("std::list::last", HandlerFn::eager(last, 1)),
        (
            "std::list::for_each",
            HandlerFn::into_function_entry(for_each, vec![Eager, Lazy]),
        ),
        (
            "std::list::map",
            HandlerFn::into_function_entry(map, vec![Eager, Lazy]),
        ),
        ("std::list::push", HandlerFn::eager(push, 2)),
        ("std::list::pop", HandlerFn::eager(pop, 1)),
        ("std::list::remove", HandlerFn::eager(remove, 2)),
        ("std::list::is_empty", HandlerFn::eager(is_empty, 1)),
        ("std::list::size", HandlerFn::eager(size, 1)),
        ("std::list::index_of", HandlerFn::eager(index_of, 2)),
        ("std::list::to_unique", HandlerFn::eager(to_unique, 1)),
        (
            "std::list::sort",
            HandlerFn::into_function_entry(sort, vec![Eager, Lazy]),
        ),
        (
            "std::list::sort_reverse",
            HandlerFn::into_function_entry(sort_reverse, vec![Eager, Lazy]),
        ),
        ("std::list::reverse", HandlerFn::eager(reverse, 1)),
        ("std::list::flat", HandlerFn::eager(flat, 1)),
        ("std::list::min", HandlerFn::eager(min, 1)),
        ("std::list::max", HandlerFn::eager(max, 1)),
        ("std::list::sum", HandlerFn::eager(sum, 1)),
        ("std::list::join", HandlerFn::eager(join, 2)),
    ]
}

fn as_list(value: &Value, err: &'static str) -> Result<ListValue, RuntimeError> {
    match value.kind.clone().unwrap_or(Kind::NullValue(0)) {
        Kind::ListValue(lv) => Ok(lv),
        _ => Err(RuntimeError::simple_str("InvalidArgumentRuntimeError", err)),
    }
}

fn as_bool(value: &Value) -> Result<bool, RuntimeError> {
    match value.kind.clone().unwrap_or(Kind::NullValue(0)) {
        Kind::BoolValue(b) => Ok(b),
        _ => Err(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected boolean result from predicate",
        )),
    }
}

fn at(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    // array, index
    args!(args => array: ListValue, index: f64);

    if index < 0.0 {
        return Signal::Failure(RuntimeError::simple_str(
            "IndexOutOfBoundsRuntimeError",
            "Negative index",
        ));
    }
    let i = index as usize;
    match array.values.get(i) {
        Some(item) => Signal::Success(item.clone()),
        None => Signal::Failure(RuntimeError::simple(
            "IndexOutOfBoundsRuntimeError",
            format!("Index {} out of bounds (len={})", i, array.values.len()),
        )),
    }
}

fn concat(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => lhs_v: Value, rhs_v: Value);

    let Kind::ListValue(lhs) = lhs_v.kind.clone().ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two arrays as arguments but received lhs={:?}",
                lhs_v
            ),
        ));
    };
    let Kind::ListValue(rhs) = rhs_v.kind.clone().ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two arrays as arguments but received rhs={:?}",
                rhs_v
            ),
        ));
    };

    let mut result = lhs.values.clone();
    result.extend(rhs.values.iter().cloned());

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: result })),
    })
}

fn filter(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(predicate_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "filter expects (array: eager, predicate: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<Value> = Vec::new();
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };
    for item in array.values.iter() {
        ctx.insert_input_type(input_type, item.clone());
        let pred_sig = run(*predicate_node, ctx);

        match pred_sig {
            Signal::Success(v) => match as_bool(&v) {
                Ok(true) => out.push(item.clone()),
                Ok(false) => {}
                Err(e) => return Signal::Failure(e),
            },
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                return other;
            }
        }
    }

    ctx.clear_input_type(input_type);
    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: out })),
    })
}

fn find(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(predicate_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "find expects (array: eager, predicate: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    for item in array.values.iter() {
        ctx.insert_input_type(input_type, item.clone());
        let pred_sig = run(*predicate_node, ctx);
        match pred_sig {
            Signal::Success(v) => match as_bool(&v) {
                Ok(true) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Success(item.clone());
                }
                Ok(false) => continue,
                Err(e) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Failure(e);
                }
            },
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }

    ctx.clear_input_type(input_type);
    Signal::Failure(RuntimeError::simple_str(
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}
fn find_last(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(predicate_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "find_last expects (array: eager, predicate: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let mut array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    array.values.reverse();
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    for item in array.values.into_iter() {
        ctx.insert_input_type(input_type, item.clone());
        let pred_sig = run(*predicate_node, ctx);
        match pred_sig {
            Signal::Success(v) => match as_bool(&v) {
                Ok(true) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Success(item);
                }
                Ok(false) => continue,
                Err(e) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Failure(e);
                }
            },
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }
    ctx.clear_input_type(input_type);
    Signal::Failure(RuntimeError::simple_str(
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}

fn find_index(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(predicate_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "find_index expects (array: eager, predicate: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    for (idx, item) in array.values.iter().enumerate() {
        ctx.insert_input_type(input_type, item.clone());
        let pred_sig = run(*predicate_node, ctx);

        match pred_sig {
            Signal::Success(v) => match as_bool(&v) {
                Ok(true) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Success(Value {
                        kind: Some(Kind::NumberValue(idx as f64)),
                    });
                }
                Ok(false) => continue,
                Err(e) => {
                    ctx.clear_input_type(input_type);
                    return Signal::Failure(e);
                }
            },
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }

    ctx.clear_input_type(input_type);
    Signal::Failure(RuntimeError::simple_str(
        "NotFoundError",
        "No item found that satisfies the predicate",
    ))
}
fn first(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array: ListValue);

    match array.values.first() {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

fn last(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array: ListValue);
    match array.values.last() {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

fn for_each(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(transform_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "map expects (array: eager, transform: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    for item in array.values.iter() {
        ctx.insert_input_type(input_type, item.clone());
        let sig = run(*transform_node, ctx);

        match sig {
            Signal::Success(_) => {}
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                return other;
            }
        }
    }

    ctx.clear_input_type(input_type);
    Signal::Success(Value {
        kind: Some(Kind::NullValue(0)),
    })
}

fn map(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(transform_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "map expects (array: eager, transform: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<Value> = Vec::with_capacity(array.values.len());
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    for item in array.values.iter() {
        ctx.insert_input_type(input_type, item.clone());
        let sig = run(*transform_node, ctx);
        match sig {
            Signal::Success(v) => out.push(v),
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }

    ctx.clear_input_type(input_type);
    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: out })),
    })
}

fn push(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected first argument to be an array",
        ));
    };
    array.values.push(item);
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

fn pop(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    array.values.pop();
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

fn remove(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
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
        Signal::Failure(RuntimeError::simple(
            "ValueNotFoundRuntimeError",
            format!("Item {:?} not found in array", item),
        ))
    }
}

fn is_empty(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(array.values.is_empty())),
    })
}

fn size(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(array.values.len() as f64)),
    })
}

fn index_of(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value, item: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected first argument to be an array",
        ));
    };

    match array.values.iter().position(|x| *x == item) {
        Some(i) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(i as f64)),
        }),
        None => Signal::Failure(RuntimeError::simple(
            "ValueNotFoundRuntimeError",
            format!("Item {:?} not found in array", item),
        )),
    }
}

fn to_unique(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
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

fn sort(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(transform_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "map expects (array: eager, transform: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let mut array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<f64> = Vec::new();
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    let input_type_next = InputType {
        node_id,
        parameter_index: 1,
        input_index: 1,
    };

    let mut signals = Vec::new();
    array.values.sort_by(|a, b| {
        ctx.insert_input_type(input_type, a.clone());
        ctx.insert_input_type(input_type_next, b.clone());
        let sig = run(*transform_node, ctx);
        signals.push(sig);
        Ordering::Equal
    });

    for sig in signals {
        match sig {
            Signal::Success(v) => {
                if let Value {
                    kind: Some(Kind::NumberValue(i)),
                } = v
                {
                    out.push(i);
                } else {
                    ctx.clear_input_type(input_type);
                    return Signal::Failure(RuntimeError::simple(
                        "InvalidArgumentRuntimeError",
                        format!(
                            "expected return value of comparator to be a number but was {:?}",
                            v
                        ),
                    ));
                }
            }
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }

    let mut i = 0usize;
    array.values.sort_by(|_, _| {
        let comp = *out.get(i).unwrap_or(&0.0);
        i += 1;
        match comp {
            n if n < 0.0 => Ordering::Less,
            0.0 => Ordering::Equal,
            _ => Ordering::Greater,
        }
    });

    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

fn sort_reverse(
    args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    let [Argument::Eval(array_v), Argument::Thunk(transform_node)] = args else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "map expects (array: eager, transform: lazy thunk), got {:?}",
                args
            ),
        ));
    };

    let mut array = match as_list(array_v, "Expected first argument to be an array") {
        Ok(a) => a,
        Err(e) => return Signal::Failure(e),
    };

    let mut out: Vec<f64> = Vec::new();
    let node_id = ctx.get_current_node_id();
    let input_type = InputType {
        node_id,
        parameter_index: 1,
        input_index: 0,
    };

    let input_type_next = InputType {
        node_id,
        parameter_index: 1,
        input_index: 1,
    };

    let mut signals = Vec::new();
    array.values.sort_by(|a, b| {
        ctx.insert_input_type(input_type, a.clone());
        ctx.insert_input_type(input_type_next, b.clone());
        let sig = run(*transform_node, ctx);
        signals.push(sig);
        Ordering::Equal
    });

    for sig in signals {
        match sig {
            Signal::Success(v) => {
                if let Value {
                    kind: Some(Kind::NumberValue(i)),
                } = v
                {
                    out.push(i);
                } else {
                    ctx.clear_input_type(input_type);
                    return Signal::Failure(RuntimeError::simple(
                        "InvalidArgumentRuntimeError",
                        format!(
                            "expected return value of comparator to be a number but was {:?}",
                            v
                        ),
                    ));
                }
            }
            other
            @ (Signal::Failure(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => {
                ctx.clear_input_type(input_type);
                return other;
            }
        }
    }

    array.values.reverse(); // keep behavior consistent with original

    let mut i = 0usize;
    array.values.sort_by(|_, _| {
        let comp = *out.get(i).unwrap_or(&0.0);
        i += 1;
        match comp {
            n if n < 0.0 => Ordering::Less,
            0.0 => Ordering::Equal,
            _ => Ordering::Greater,
        }
    });

    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

fn reverse(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(mut array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected an array as an argument",
        ));
    };
    array.values.reverse();
    Signal::Success(Value {
        kind: Some(Kind::ListValue(array)),
    })
}

fn flat(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array_v: Value);
    let Kind::ListValue(array) = array_v.kind.ok_or(()).unwrap_or(Kind::NullValue(0)) else {
        return Signal::Failure(RuntimeError::simple_str(
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

fn min(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array: ListValue);

    let mut nums: Vec<f64> = Vec::new();
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = v.kind {
            nums.push(n);
        }
    }

    match nums.iter().min_by(|a, b| a.total_cmp(b)) {
        Some(m) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(*m)),
        }),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "Array is empty",
        )),
    }
}

fn max(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array: ListValue);

    let mut nums: Vec<f64> = Vec::new();
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = v.kind {
            nums.push(n);
        }
    }

    match nums.iter().max_by(|a, b| a.total_cmp(b)) {
        Some(m) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(*m)),
        }),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "Array is empty",
        )),
    }
}

fn sum(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => array: ListValue);

    let mut s = 0.0;
    for v in &array.values {
        if let Some(Kind::NumberValue(n)) = v.kind {
            s += n;
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(s)),
    })
}

fn join(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
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
    use crate::context::context::Context;
    use tucana::shared::{ListValue, Value, value::Kind};

    // --- helpers -------------------------------------------------------------
    fn a_val(v: Value) -> Argument {
        Argument::Eval(v)
    }
    fn v_num(n: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(n)),
        }
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

    fn expect_num(sig: Signal) -> f64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => n,
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

    fn dummy_run(_: i64, _: &mut Context) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    // --- at ------------------------------------------------------------------
    #[test]
    fn test_at_success() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let a = v_list(vec![v_num(1.0), v_num(2.0)]);
        let b = v_list(vec![v_num(3.0), v_num(4.0)]);
        let out = expect_list(concat(&[a_val(a), a_val(b)], &mut ctx, &mut run));
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].kind, Some(Kind::NumberValue(1.0)));
        assert_eq!(out[3].kind, Some(Kind::NumberValue(4.0)));
    }

    #[test]
    fn test_concat_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let array = v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]);
        let predicate = v_list(vec![v_bool(true), v_bool(false), v_bool(true)]);
        let out = expect_list(filter(
            &[a_val(array), a_val(predicate)],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, Some(Kind::NumberValue(1.0)));
        assert_eq!(out[1].kind, Some(Kind::NumberValue(3.0)));
    }

    #[test]
    fn test_filter_error() {
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let array = v_list(vec![v_num(1.0)]);
        let predicate = v_list(vec![v_bool(true)]);
        match filter(&[a_val(array.clone())], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            x => panic!("{:?}", x),
        }
        match filter(
            &[a_val(v_str("not_array")), a_val(predicate.clone())],
            &mut ctx,
            &mut run,
        ) {
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_str("first"), v_str("second"), v_str("third")]);
        assert_eq!(
            expect_str(first(&[a_val(arr)], &mut ctx, &mut run)),
            "first"
        );
    }

    #[test]
    fn test_first_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let arr = v_list(vec![v_str("first"), v_str("second"), v_str("last")]);
        assert_eq!(expect_str(last(&[a_val(arr)], &mut ctx, &mut run)), "last");
    }

    #[test]
    fn test_last_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        match for_each(&[], &mut ctx, &mut run) {
            Signal::Success(Value {
                kind: Some(Kind::NullValue(_)),
            }) => {}
            x => panic!("expected NullValue, got {:?}", x),
        }
        let transformed = v_list(vec![v_str("X"), v_str("Y")]);
        let out = expect_list(map(
            &[
                a_val(v_list(vec![v_num(1.0), v_num(2.0)])),
                a_val(transformed.clone()),
            ],
            &mut ctx,
            &mut run,
        ));
        let expected = match transformed.kind {
            Some(Kind::ListValue(ListValue { values })) => values,
            _ => unreachable!(),
        };
        assert_eq!(out, expected);
    }

    // --- push / pop / remove -------------------------------------------------
    #[test]
    fn test_push_success() {
        let mut ctx = Context::default();
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
        assert_eq!(out[2].kind, Some(Kind::NumberValue(3.0)));
    }

    #[test]
    fn test_push_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let out = expect_list(pop(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]))],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, Some(Kind::NumberValue(1.0)));
        assert_eq!(out[1].kind, Some(Kind::NumberValue(2.0)));
    }

    #[test]
    fn test_pop_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
        assert_eq!(uniq[0].kind, Some(Kind::NumberValue(10.0)));
        assert_eq!(uniq[1].kind, Some(Kind::NumberValue(42.0)));
        assert_eq!(uniq[2].kind, Some(Kind::NumberValue(30.0)));
    }

    #[test]
    fn test_index_of_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
        let mut run = dummy_run;

        // We don't rely on actual values; ordering is driven by the comparator sequence.
        let arr = v_list(vec![v_str("a"), v_str("b"), v_str("c"), v_str("d")]);
        let comps = v_list(vec![v_num(-1.0), v_num(1.0), v_num(0.0), v_num(-1.0)]);
        let out = expect_list(sort(
            &[a_val(arr.clone()), a_val(comps.clone())],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out.len(), 4);

        let out_r = expect_list(sort_reverse(
            &[a_val(arr), a_val(comps)],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out_r.len(), 4);
    }

    // --- reverse / flat ------------------------------------------------------
    #[test]
    fn test_reverse_success_and_error() {
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let out = expect_list(reverse(
            &[a_val(v_list(vec![v_num(1.0), v_num(2.0), v_num(3.0)]))],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(out[0].kind, Some(Kind::NumberValue(3.0)));
        assert_eq!(out[2].kind, Some(Kind::NumberValue(1.0)));

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
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let nested = v_list(vec![
            v_num(1.0),
            v_list(vec![v_num(2.0), v_num(3.0)]),
            v_list(vec![]),
            v_num(4.0),
        ]);
        let out = expect_list(flat(&[a_val(nested)], &mut ctx, &mut run));
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].kind, Some(Kind::NumberValue(1.0)));
        assert_eq!(out[1].kind, Some(Kind::NumberValue(2.0)));
        assert_eq!(out[2].kind, Some(Kind::NumberValue(3.0)));
        assert_eq!(out[3].kind, Some(Kind::NumberValue(4.0)));
    }

    // --- min / max / sum -----------------------------------------------------
    #[test]
    fn test_min_max_sum_success_and_error() {
        let mut ctx = Context::default();
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
        let mut ctx = Context::default();
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
