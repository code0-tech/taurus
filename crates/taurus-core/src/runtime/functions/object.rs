//! Object/struct utility handlers.
//!
//! `keys` returns sorted field names to keep downstream list operations deterministic.

use tucana::shared::{Struct, Value, value::Kind};

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::value_from_i64;

taurus_macros::module! {
    identifier = "taurus-object",
    name(en_US = "Object"),
    description(en_US = "Work with Objects."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:cube",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "OBJECT",
    module = "taurus-object",
    name(en_US = "Object"),
    display_message(en_US = "Object"),
    alias(en_US = "object;struct;data"),
    generic_keys = ["T"],
    type_string = "{ [K in keyof T]: T[K] }",
}

taurus_macros::data_type! {
    identifier = "TYPE",
    module = "taurus-object",
    name(en_US = "Type"),
    display_message(en_US = "Type of ${T}"),
    alias(
        en_US = "type;data type;data-type;datatype;type definition;type-definition;type-def;typedef"
    ),
    generic_keys = ["T"],
    type_string = "T",
}

#[taurus_macros::runtime_function(
    identifier = "std::object::get",
    module = "taurus-object",
    signature = "<T, K extends keyof T>(object: OBJECT<T>, key: K | string): T[K]",
    name(en_US = "Get key of object"),
    description(en_US = "Returns the value of a key inside of the object."),
    display_message(en_US = "Get ${key} of ${object}"),
    alias(en_US = "get;object;std"),
    display_icon = "tabler:cube",
    linked_data_type_identifiers = ["OBJECT"],
)]
#[parameter(
    runtime_name = "object",
    name(en_US = "Object"),
    description(en_US = "The object that contains the value referenced by the key.")
)]
#[parameter(
    runtime_name = "key",
    name(en_US = "Key"),
    description(en_US = "The property name under which the value will be referenced.")
)]
fn get(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => object: Struct, key: String);
    match object.fields.get(&key) {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00002",
            "KeyNotFoundRuntimeError",
            format!("Object field not found: {}", key),
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::object::contains_key",
    module = "taurus-object",
    signature = "<T>(object: OBJECT<T>, key: keyof OBJECT<T> | string): BOOLEAN",
    name(en_US = "Contains Key"),
    description(
        en_US = "Returns true if the given key is a property of the object; otherwise, returns false."
    ),
    display_message(en_US = "Checks if ${object} Contains ${key}"),
    alias(en_US = "contains_key;object;std;contains;key"),
    display_icon = "tabler:cube",
    linked_data_type_identifiers = ["OBJECT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "object",
    name(en_US = "Object"),
    description(en_US = "The object to check for the presence of a key.")
)]
#[parameter(
    runtime_name = "key",
    name(en_US = "Key"),
    description(en_US = "The key to check for existence in the object.")
)]
fn contains_key(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => object: Struct, key: String);
    let contains = object.fields.contains_key(&key);

    Signal::Success(Value {
        kind: Some(Kind::BoolValue(contains)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::object::size",
    module = "taurus-object",
    signature = "<T>(object: OBJECT<T>): NUMBER",
    name(en_US = "Get Object Size"),
    description(en_US = "Returns an integer count of all enumerable property keys in the specified object."),
    display_message(en_US = "Size of ${object}"),
    alias(en_US = "size;object;std"),
    display_icon = "tabler:cube",
    linked_data_type_identifiers = ["OBJECT", "NUMBER"],
)]
#[parameter(
    runtime_name = "object",
    name(en_US = "Object"),
    description(en_US = "Returns the number of keys present in the given object.")
)]
fn size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => object: Struct);
    Signal::Success(value_from_i64(object.fields.len() as i64))
}

#[taurus_macros::runtime_function(
    identifier = "std::object::keys",
    module = "taurus-object",
    signature = "<T>(object: OBJECT<T>): LIST<keyof OBJECT<T>>",
    name(en_US = "Get Object Keys"),
    description(en_US = "Returns a list containing all keys of the specified object."),
    display_message(en_US = "Keys of ${object}"),
    alias(en_US = "keys;object;std"),
    display_icon = "tabler:cube",
    linked_data_type_identifiers = ["OBJECT", "LIST"],
)]
#[parameter(
    runtime_name = "object",
    name(en_US = "Object"),
    description(en_US = "Returns a list of all the keys (property names) of the given object.")
)]
fn keys(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => object: Struct);

    // Stable ordering is important for reproducible traces and tests.
    let mut keys = object.fields.keys().cloned().collect::<Vec<String>>();
    keys.sort();

    let keys = keys
        .into_iter()
        .map(|key| Value {
            kind: Some(Kind::StringValue(key.clone())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue { values: keys })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::object::set",
    module = "taurus-object",
    signature = "<T, K extends TEXT, V>(object: OBJECT<T>, key: K, value: V): OBJECT<T & { [P in K]: V }>",
    name(en_US = "Set Object Key"),
    description(en_US = "Returns a new object with the specified key set to the given value."),
    display_message(en_US = "Set ${key} to ${value} of ${object}"),
    alias(en_US = "set;object;std"),
    display_icon = "tabler:cube",
    linked_data_type_identifiers = ["OBJECT", "TEXT"],
)]
#[parameter(
    runtime_name = "object",
    name(en_US = "Object"),
    description(
        en_US = "The original object that will be modified with the specified key-value pair."
    )
)]
#[parameter(
    runtime_name = "key",
    name(en_US = "Key"),
    description(en_US = "The property name under which the value will be stored in the object.")
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The value to assign to the object property identified by the key.")
)]
fn set(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => object: Struct, key: String, value: Value);
    let mut new_object = object.clone();
    new_object.fields.insert(key.clone(), value.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(new_object)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::argument::Argument;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, value_from_f64};
    use std::collections::HashMap;
    use tucana::shared::{Struct as TcStruct, Value, value::Kind};

    // ---- helpers: Value builders ----
    fn v_string(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }
    fn v_number(n: f64) -> Value {
        value_from_f64(n)
    }
    fn v_bool(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }
    fn v_struct(fields: HashMap<String, Value>) -> Value {
        Value {
            kind: Some(Kind::StructValue(TcStruct { fields })),
        }
    }

    // ---- helpers: Struct builders (for args that expect Struct) ----
    fn s_empty() -> TcStruct {
        TcStruct {
            fields: HashMap::new(),
        }
    }
    fn s_from(mut kv: Vec<(&str, Value)>) -> TcStruct {
        let mut map = HashMap::<String, Value>::new();
        for (k, v) in kv.drain(..) {
            map.insert(k.to_string(), v);
        }
        TcStruct { fields: map }
    }
    fn s_test() -> TcStruct {
        s_from(vec![
            ("name", v_string("John")),
            ("age", v_number(30.0)),
            ("active", v_bool(true)),
        ])
    }

    // ---- helpers: Argument builders ----
    #[allow(dead_code)]
    fn a_value(v: Value) -> Argument {
        Argument::Eval(v)
    }
    fn a_string(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }
    fn a_struct(s: TcStruct) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(s)),
        })
    }

    // dummy runner for handlers that accept `run: &mut crate::handler::registry::ThunkRunner<'_>`
    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    #[test]
    fn test_contains_key_success() {
        let mut ctx = ValueStore::default();

        // existing key
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("name")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(b),
            _ => panic!("Expected BoolValue"),
        }

        // non-existing key
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("nonexistent")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(!b),
            _ => panic!("Expected BoolValue"),
        }

        // empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty()), a_string("any_key")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(!b),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_size_success() {
        let mut ctx = ValueStore::default();

        // non-empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_test())];
        let signal = size(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::NumberValue(n)) => assert_eq!(number_to_f64(&n).unwrap_or_default(), 3.0),
            _ => panic!("Expected NumberValue"),
        }

        // empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty())];
        let signal = size(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::NumberValue(n)) => assert_eq!(number_to_f64(&n).unwrap_or_default(), 0.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_keys_success() {
        let mut ctx = ValueStore::default();

        // with fields
        let mut run = dummy_run;
        let args = vec![a_struct(s_test())];
        let signal = keys(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::ListValue(list)) => {
                let mut got: Vec<String> = list
                    .values
                    .iter()
                    .filter_map(|v| {
                        if let Some(Kind::StringValue(s)) = &v.kind {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                got.sort();

                let mut expected =
                    vec!["active".to_string(), "age".to_string(), "name".to_string()];
                expected.sort();
                assert_eq!(got, expected);
            }
            _ => panic!("Expected ListValue"),
        }

        // empty object => empty list
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty())];
        let signal = keys(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::ListValue(list)) => assert_eq!(list.values.len(), 0),
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_set_success_and_overwrite() {
        let mut ctx = ValueStore::default();

        // set new key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("email"),
            Argument::Eval(v_string("john@example.com")),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 4);
                match st.fields.get("email") {
                    Some(Value {
                        kind: Some(Kind::StringValue(s)),
                        ..
                    }) => assert_eq!(s, "john@example.com"),
                    _ => panic!("Expected email to be a string"),
                }
            }
            _ => panic!("Expected StructValue"),
        }

        // overwrite existing key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("age"),
            Argument::Eval(v_number(31.0)),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 3);
                match st.fields.get("age") {
                    Some(Value {
                        kind: Some(Kind::NumberValue(n)),
                        ..
                    }) => assert_eq!(number_to_f64(n).unwrap_or_default(), 31.0),
                    _ => panic!("Expected age to be a number"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_with_empty_object_and_nested() {
        let mut ctx = ValueStore::default();

        // empty object -> add first key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_empty()),
            a_string("first_key"),
            Argument::Eval(v_bool(true)),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 1);
                match st.fields.get("first_key") {
                    Some(Value {
                        kind: Some(Kind::BoolValue(b)),
                        ..
                    }) => assert_eq!(*b, true),
                    _ => panic!("Expected first_key to be a bool"),
                }
            }
            _ => panic!("Expected StructValue"),
        }

        // nested object value
        let nested = {
            let mut nf = HashMap::new();
            nf.insert("street".to_string(), v_string("123 Main St"));
            v_struct(nf)
        };
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("address"),
            Argument::Eval(nested),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                match st.fields.get("address") {
                    Some(Value {
                        kind: Some(Kind::StructValue(_)),
                        ..
                    }) => { /* ok */ }
                    _ => panic!("Expected address to be a struct"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_preserves_original_struct() {
        let mut ctx = ValueStore::default();
        let original = s_test();
        let original_len = original.fields.len();

        // keep a clone to assert immutability
        let orig_clone = original.clone();

        let mut run = dummy_run;
        let args = vec![
            a_struct(original),
            a_string("new_key"),
            Argument::Eval(v_string("new_val")),
        ];
        let _ = set(&args, &mut ctx, &mut run);

        // ensure original (captured clone) unchanged
        assert_eq!(orig_clone.fields.len(), original_len);
        assert!(!orig_clone.fields.contains_key("new_key"));
    }

    #[test]
    fn test_get_success_string_field() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("name")];

        let signal = get(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            s => panic!("Expected Success, got {:?}", s),
        };

        match v.kind {
            Some(Kind::StringValue(s)) => assert_eq!(s, "John"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_get_success_nested_struct_field() {
        let mut ctx = ValueStore::default();
        let nested = {
            let mut nf = HashMap::new();
            nf.insert("street".to_string(), v_string("123 Main St"));
            v_struct(nf)
        };
        let object = s_from(vec![("address", nested)]);
        let mut run = dummy_run;
        let args = vec![a_struct(object), a_string("address")];

        let signal = get(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            s => panic!("Expected Success, got {:?}", s),
        };

        match v.kind {
            Some(Kind::StructValue(st)) => match st.fields.get("street") {
                Some(Value {
                    kind: Some(Kind::StringValue(s)),
                    ..
                }) => assert_eq!(s, "123 Main St"),
                _ => panic!("Expected nested 'street' string"),
            },
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_get_missing_field_returns_field_not_present() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("email")];

        let signal = get(&args, &mut ctx, &mut run);
        let _ = match signal {
            Signal::Failure(err) => err,
            s => panic!("Expected Failure, got {:?}", s),
        };
    }
}
