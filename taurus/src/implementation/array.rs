use std::cmp::Ordering;

use tucana::shared::{Value, value::Kind};

use crate::context::signal::Signal;
use crate::{context::Context, error::RuntimeError, };
use crate::context::registry::HandlerFn;

pub fn collect_array_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::array::at", at),
        ("std::array::concat", concat),
        ("std::array::filter", filter),
        ("std::array::find", find),
        ("std::array::find_last", find_last),
        ("std::array::find_index", find_index),
        ("std::array::first", first),
        ("std::array::last", last),
        ("std::array::for_each", for_each),
        ("std::array::map", map),
        ("std::array::push", push),
        ("std::array::pop", pop),
        ("std::array::remove", remove),
        ("std::array::is_empty", is_empty),
        ("std::array::size", size),
        ("std::array::index_of", index_of),
        ("std::array::to_unique", to_unique),
        ("std::array::sort", sort),
        ("std::array::sort_reverse", sort_reverse),
        ("std::array::reverse", reverse),
        ("std::array::flat", flat),
        ("std::array::min", min),
        ("std::array::max", max),
        ("std::array::sum", sum),
        ("std::array::join", join),
    ]
}

fn at(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::NumberValue(index)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected a list and a number as arguments but received {:?}",
                values
            ),
        ));
    };

    let item = array.values[*index as usize].clone();

    Signal::Success(item)
}

fn concat(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(lhs)),
        },
        Value {
            kind: Some(Kind::ListValue(rhs)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut result = lhs.values.clone();
    result.extend(rhs.values.clone());

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: result,
        })),
    })
}

fn filter(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_predicate)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut predicate_collector = vec![];
    resolved_predicate.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::BoolValue(pred)),
        } = v.clone()
        {
            predicate_collector.push(pred);
        }
    });

    let mut index = 0;
    let new_array = array
        .values
        .clone()
        .into_iter()
        .filter(|_| {
            let predicate = predicate_collector[index];
            index += 1;
            predicate
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: new_array,
        })),
    })
}

fn find(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_predicate)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut predicate_collector = vec![];
    resolved_predicate.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::BoolValue(pred)),
        } = v.clone()
        {
            predicate_collector.push(pred);
        }
    });

    let mut index = 0;
    let item = array.values.clone().into_iter().find(|_| {
        let predicate = predicate_collector[index];
        index += 1;
        predicate
    });

    match item {
        Some(item) => Signal::Success(item),
        None => Signal::Failure(RuntimeError::simple(
            "NotFoundError",
            "No item found that satisfies the predicate".to_string(),
        )),
    }
}

fn find_last(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_predicate)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut predicate_collector = vec![];
    resolved_predicate.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::BoolValue(pred)),
        } = v.clone()
        {
            predicate_collector.push(pred);
        }
    });

    let mut index = 0;
    let mut new_array = array.values.clone();
    new_array.reverse();

    let item = new_array.into_iter().find(|_| {
        let predicate = predicate_collector[index];
        index += 1;
        predicate
    });

    match item {
        Some(item) => Signal::Success(item),
        None => Signal::Failure(RuntimeError::simple(
            "NotFoundError",
            "No item found that satisfies the predicate".to_string(),
        )),
    }
}

fn find_index(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_predicate)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut predicate_collector = vec![];
    resolved_predicate.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::BoolValue(pred)),
        } = v.clone()
        {
            predicate_collector.push(pred);
        }
    });

    let mut index = 0;
    let new_array = array.values.clone();

    let item = new_array.into_iter().find(|_| {
        let predicate = predicate_collector[index];

        if !predicate {
            index += 1;
        }
        predicate
    });

    match item {
        Some(_) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(index as f64)),
        }),
        None => Signal::Failure(RuntimeError::simple(
            "NotFoundError",
            "No item found that satisfies the predicate".to_string(),
        )),
    }
}

fn first(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    match array.values.first() {
        Some(item) => Signal::Success(item.clone()),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

fn last(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    match array.values.last() {
        Some(item) => Signal::Success(item.clone()),
        None => Signal::Failure(RuntimeError::simple_str(
            "ArrayEmptyRuntimeError",
            "This array is empty",
        )),
    }
}

/*
 * for_each has no implementation
 *
 * Reason:
 * The definition itself takes in an array and a node
 * The node itself will be executed on the arrays elements
 * If the node is (CONSUMER) resolved it goes in this function --> therefor all code is already executed
 */
fn for_each(_values: &[Value], _ctx: &mut Context) -> Signal {
    Signal::Success(Value {
        kind: Some(Kind::NullValue(0)),
    })
}

/*
 * Same as for_each
 *
 * Reason:
 * The definition itself takes in an array and a node
 * The node itself will be executed on the arrays elements
 * If the node is (TRANSFORM) resolved it goes in this function --> therefor all code is already executed
 * The TRANSFORM node is gives a new item as a result
 */
fn map(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(_array)),
        },
        Value {
            kind: Some(Kind::ListValue(transform_result)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    Signal::Success(Value {
        kind: Some(Kind::ListValue(transform_result.clone())),
    })
}

fn push(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        item,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut new_array = array.clone();
    new_array.values.push(item.clone());
    Signal::Success(Value {
        kind: Some(Kind::ListValue(new_array)),
    })
}

fn pop(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut new_array = array.clone();
    new_array.values.pop();
    Signal::Success(Value {
        kind: Some(Kind::ListValue(new_array)),
    })
}

fn remove(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        item,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut new_array = array.clone();
    let item_clone = item.clone();
    let index = match new_array.values.iter().position(|x| *x == item_clone) {
        Some(index) => index,
        None => {
            return Signal::Failure(RuntimeError::simple(
                "ValueNotFoundRuntimeError",
                format!("Item {:?} not found in array", item_clone),
            ));
        }
    };

    new_array.values.remove(index);

    Signal::Success(Value {
        kind: Some(Kind::ListValue(new_array)),
    })
}

fn is_empty(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    Signal::Success(Value {
        kind: Some(Kind::BoolValue(array.values.is_empty())),
    })
}

fn size(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(array.values.len() as f64)),
    })
}

fn index_of(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        item,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let new_array = array.clone();
    let item_clone = item.clone();
    let index = match new_array.values.iter().position(|x| *x == item_clone) {
        Some(index) => index,
        None => {
            return Signal::Failure(RuntimeError::simple(
                "ValueNotFoundRuntimeError",
                format!("Item {:?} not found in array", item_clone),
            ));
        }
    };

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(index as f64)),
    })
}

fn to_unique(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut unique_values = Vec::new();

    for value in &array.values {
        if !unique_values.contains(value) {
            unique_values.push(value.clone());
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: unique_values,
        })),
    })
}

fn sort(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_comparator)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut comparate_collector = vec![];
    resolved_comparator.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::NumberValue(comp)),
        } = v.clone()
        {
            comparate_collector.push(comp);
        }
    });

    let mut index = 0;
    let mut new_array = array.values.clone();
    new_array.sort_by(|_, _| {
        let comp = comparate_collector[index];
        index += 1;
        match comp {
            -1.0 => Ordering::Less,
            0.0 => Ordering::Equal,
            _ => Ordering::Greater,
        }
    });

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: new_array,
        })),
    })
}

fn sort_reverse(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::ListValue(resolved_comparator)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected two arrays as arguments but received {:?}", values),
        ));
    };

    let mut comparate_collector = vec![];
    resolved_comparator.values.iter().for_each(|v| {
        if let Value {
            kind: Some(Kind::NumberValue(comp)),
        } = v.clone()
        {
            comparate_collector.push(comp);
        }
    });

    let mut index = 0;
    let mut new_array = array.values.clone();
    new_array.reverse();

    new_array.sort_by(|_, _| {
        let comp = comparate_collector[index];
        index += 1;
        match comp {
            -1.0 => Ordering::Less,
            0.0 => Ordering::Equal,
            _ => Ordering::Greater,
        }
    });

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: new_array,
        })),
    })
}

fn reverse(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut new_array = array.values.clone();
    new_array.reverse();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: new_array,
        })),
    })
}

fn flat(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected an array as an argument but received {:?}", values),
        ));
    };

    let mut flattable_array: Vec<Vec<Value>> = Vec::new();

    for item in &array.values {
        if let Value {
            kind: Some(Kind::ListValue(sub_array)),
        } = item
        {
            flattable_array.push(sub_array.values.clone());
        } else {
            flattable_array.push(vec![item.clone()]);
        }
    }

    let flattend = flattable_array
        .into_iter()
        .flatten()
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue {
            values: flattend,
        })),
    })
}

fn min(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an array of numbers as an argument but received {:?}",
                values
            ),
        ));
    };

    let mut new_array = vec![];
    array.values.clone().into_iter().for_each(|a| {
        if let Value {
            kind: Some(Kind::NumberValue(number)),
        } = a
        {
            new_array.push(number);
        }
    });

    let min = new_array.iter().min_by(|a, b| a.total_cmp(b));

    match min {
        Some(min) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(*min)),
        }),
        None => Signal::Failure(RuntimeError::simple(
            "ArrayEmptyRuntimeError",
            "Array is empty".to_string(),
        )),
    }
}

fn max(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an array of numbers as an argument but received {:?}",
                values
            ),
        ));
    };

    let mut new_array = vec![];
    array.values.clone().into_iter().for_each(|a| {
        if let Value {
            kind: Some(Kind::NumberValue(number)),
        } = a
        {
            new_array.push(number);
        }
    });

    let max = new_array.iter().max_by(|a, b| a.total_cmp(b));

    match max {
        Some(max) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(*max)),
        }),
        None => Signal::Failure(RuntimeError::simple(
            "ArrayEmptyRuntimeError",
            "Array is empty".to_string(),
        )),
    }
}

fn sum(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an array of numbers as an argument but received {:?}",
                values
            ),
        ));
    };

    let mut sum = 0.0;
    array.values.clone().into_iter().for_each(|a| {
        if let Value {
            kind: Some(Kind::NumberValue(number)),
        } = a
        {
            sum += number;
        }
    });

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(sum)),
    })
}

fn join(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::ListValue(array)),
        },
        Value {
            kind: Some(Kind::StringValue(separator)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected array of text and a text as arguments but received {:?}",
                values
            ),
        ));
    };

    let mut collector = vec![];
    array.values.clone().into_iter().for_each(|a| {
        if let Value {
            kind: Some(Kind::StringValue(string)),
        } = a
        {
            collector.push(string);
        }
    });

    let joined = collector.join(separator);

    Signal::Success(Value {
        kind: Some(Kind::StringValue(joined)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use tucana::shared::{ListValue, Value, value::Kind};

    // Helper function to create a number value
    fn create_number_value(num: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(num)),
        }
    }

    // Helper function to create a string value
    fn create_string_value(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }

    // Helper function to create a bool value
    fn create_bool_value(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }

    // Helper function to create an array value
    fn create_array_value(values: Vec<Value>) -> Value {
        Value {
            kind: Some(Kind::ListValue(ListValue { values })),
        }
    }

    #[test]
    fn test_at_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(10.0),
            create_number_value(20.0),
            create_number_value(30.0),
        ]);

        // Test getting first element
        let values = vec![array.clone(), create_number_value(0.0)];
        let result = at(&values, &mut ctx);
        match result {
            Signal::Success(v) => match v.kind {
                Some(Kind::NumberValue(val)) => assert_eq!(val, 10.0),
                _ => panic!("Expected NumberValue"),
            },
            _ => panic!("Expected NumberValue"),
        }

        // Test getting second element
        let values = vec![array.clone(), create_number_value(1.0)];
        let signal = at(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 20.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test getting third element
        let values = vec![array, create_number_value(2.0)];
        let signal = at(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 30.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_at_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = at(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), create_number_value(0.0)];
        let result = at(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong type for second parameter
        let values = vec![array, create_string_value("not_number")];
        let result = at(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_concat_success() {
        let mut ctx = Context::new();

        let array1 = create_array_value(vec![create_number_value(1.0), create_number_value(2.0)]);
        let array2 = create_array_value(vec![create_number_value(3.0), create_number_value(4.0)]);

        let values = vec![array1, array2];
        let signal = concat(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 4);

                // Check each element
                match &list.values[0].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 1.0),
                    _ => panic!("Expected NumberValue at index 0"),
                }
                match &list.values[1].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 2.0),
                    _ => panic!("Expected NumberValue at index 1"),
                }
                match &list.values[2].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 3.0),
                    _ => panic!("Expected NumberValue at index 2"),
                }
                match &list.values[3].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 4.0),
                    _ => panic!("Expected NumberValue at index 3"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_concat_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = concat(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), array.clone()];
        let result = concat(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong type for second parameter
        let values = vec![array, create_number_value(42.0)];
        let result = concat(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_filter_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(1.0),
            create_number_value(2.0),
            create_number_value(3.0),
        ]);
        let predicate = create_array_value(vec![
            create_bool_value(true),
            create_bool_value(false),
            create_bool_value(true),
        ]);

        let values = vec![array, predicate];
        let signal = filter(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 2);

                match &list.values[0].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 1.0),
                    _ => panic!("Expected NumberValue at index 0"),
                }
                match &list.values[1].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 3.0),
                    _ => panic!("Expected NumberValue at index 1"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_filter_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);
        let predicate = create_array_value(vec![create_bool_value(true)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = filter(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), predicate.clone()];
        let result = filter(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for second parameter
        let values = vec![array, create_number_value(42.0)];
        let result = filter(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_first_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_string_value("first"),
            create_string_value("second"),
            create_string_value("third"),
        ]);

        let values = vec![array];
        let signal = first(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "first"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_first_error() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let result = first(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = first(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = first(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_last_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_string_value("first"),
            create_string_value("second"),
            create_string_value("last"),
        ]);

        let values = vec![array];
        let signal = last(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "last"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_last_error() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let result = last(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = last(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_push_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![create_number_value(1.0), create_number_value(2.0)]);
        let new_element = create_number_value(3.0);

        let values = vec![array, new_element];
        let signal = push(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 3);

                match &list.values[2].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 3.0),
                    _ => panic!("Expected NumberValue at last index"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_push_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = push(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), create_number_value(42.0)];
        let result = push(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_pop_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(1.0),
            create_number_value(2.0),
            create_number_value(3.0),
        ]);

        let values = vec![array];
        let signal = pop(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 2);

                // Check that the last element was removed
                match &list.values[0].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 1.0),
                    _ => panic!("Expected NumberValue at index 0"),
                }
                match &list.values[1].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 2.0),
                    _ => panic!("Expected NumberValue at index 1"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_pop_error() {
        let mut ctx = Context::new();

        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = pop(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = pop(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_is_empty_success() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let signal = is_empty(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test with non-empty array
        let non_empty_array = create_array_value(vec![create_number_value(1.0)]);
        let values = vec![non_empty_array];
        let signal = is_empty(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_empty_error() {
        let mut ctx = Context::new();

        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = is_empty(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = is_empty(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_size_success() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let signal = size(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 0.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test with array of 3 elements
        let array = create_array_value(vec![
            create_number_value(1.0),
            create_number_value(2.0),
            create_number_value(3.0),
        ]);
        let values = vec![array];
        let signal = size(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_size_error() {
        let mut ctx = Context::new();

        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_reverse_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(1.0),
            create_number_value(2.0),
            create_number_value(3.0),
        ]);

        let values = vec![array];
        let signal = reverse(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 3);

                // Check that elements are reversed
                match &list.values[0].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 3.0),
                    _ => panic!("Expected NumberValue at index 0"),
                }
                match &list.values[1].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 2.0),
                    _ => panic!("Expected NumberValue at index 1"),
                }
                match &list.values[2].kind {
                    Some(Kind::NumberValue(val)) => assert_eq!(*val, 1.0),
                    _ => panic!("Expected NumberValue at index 2"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_reverse_error() {
        let mut ctx = Context::new();

        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = reverse(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = reverse(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_sum_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(1.5),
            create_number_value(2.5),
            create_number_value(3.0),
        ]);

        let values = vec![array];
        let signal = sum(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 7.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_sum_empty_array() {
        let mut ctx = Context::new();

        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let signal = sum(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 0.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_sum_error() {
        let mut ctx = Context::new();

        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = sum(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong number of parameters
        let values = vec![];
        let result = sum(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_join_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_string_value("hello"),
            create_string_value("world"),
            create_string_value("test"),
        ]);
        let separator = create_string_value(", ");

        let values = vec![array, separator];
        let signal = join(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "hello, world, test"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_join_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_string_value("hello")]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = join(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), create_string_value(",")];
        let result = join(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for second parameter
        let values = vec![array, create_number_value(42.0)];
        let result = join(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_min_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(5.0),
            create_number_value(1.0),
            create_number_value(3.0),
            create_number_value(2.0),
        ]);

        let values = vec![array];
        let signal = min(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 1.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_min_error() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let result = min(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = min(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_max_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(5.0),
            create_number_value(1.0),
            create_number_value(8.0),
            create_number_value(2.0),
        ]);

        let values = vec![array];
        let signal = max(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 8.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_max_error() {
        let mut ctx = Context::new();

        // Test with empty array
        let empty_array = create_array_value(vec![]);
        let values = vec![empty_array];
        let result = max(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong parameter type
        let values = vec![create_string_value("not_array")];
        let result = max(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_index_of_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_number_value(10.0),
            create_number_value(42.0),
            create_number_value(30.0),
            create_number_value(42.0), // duplicate
        ]);
        let search_element = create_number_value(42.0);

        let values = vec![array, search_element];
        let signal = index_of(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 1.0), // Should return first occurrence
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_index_of_not_found() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_string_value("hello"),
            create_string_value("world"),
        ]);
        let search_element = create_string_value("missing");

        let values = vec![array, search_element];
        let result = index_of(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_index_of_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = index_of(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), create_number_value(42.0)];
        let result = index_of(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_remove_success() {
        let mut ctx = Context::new();

        let array = create_array_value(vec![
            create_string_value("first"),
            create_string_value("second"),
            create_string_value("third"),
        ]);
        let item_to_remove = create_string_value("second"); // Remove middle element

        let values = vec![array, item_to_remove];
        let signal = remove(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 2);

                match &list.values[0].kind {
                    Some(Kind::StringValue(val)) => assert_eq!(val, "first"),
                    _ => panic!("Expected StringValue at index 0"),
                }
                match &list.values[1].kind {
                    Some(Kind::StringValue(val)) => assert_eq!(val, "third"),
                    _ => panic!("Expected StringValue at index 1"),
                }
            }
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_remove_error() {
        let mut ctx = Context::new();
        let array = create_array_value(vec![create_number_value(1.0)]);

        // Test with wrong number of parameters
        let values = vec![array.clone()];
        let result = remove(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with wrong type for first parameter
        let values = vec![create_string_value("not_array"), create_number_value(0.0)];
        let result = remove(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
        // Test with item not found
        let values = vec![array, create_number_value(999.0)];
        let result = remove(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }
}
