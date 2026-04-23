//! Text/string standard-library handlers.
//!
//! Index semantics are explicit per operation:
//! - character-based for access-oriented operations like `at`
//! - byte-based where direct string insertion/removal APIs are used
//!   (to preserve historical runtime behavior)

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::{number_to_f64, number_to_i64_lossy, value_from_i64};
use base64::Engine;
use tucana::shared::{ListValue, Value, value::Kind};

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("std::text::as_bytes", as_bytes, 1),
    FunctionRegistration::eager("std::text::byte_size", byte_size, 1),
    FunctionRegistration::eager("std::text::capitalize", capitalize, 1),
    FunctionRegistration::eager("std::text::lowercase", lowercase, 1),
    FunctionRegistration::eager("std::text::uppercase", uppercase, 1),
    FunctionRegistration::eager("std::text::swapcase", swapcase, 1),
    FunctionRegistration::eager("std::text::trim", trim, 1),
    FunctionRegistration::eager("std::text::chars", chars, 1),
    FunctionRegistration::eager("std::text::at", at, 2),
    FunctionRegistration::eager("std::text::append", append, 2),
    FunctionRegistration::eager("std::text::prepend", prepend, 2),
    FunctionRegistration::eager("std::text::insert", insert, 3),
    FunctionRegistration::eager("std::text::length", length, 1),
    FunctionRegistration::eager("std::text::reverse", reverse, 1),
    FunctionRegistration::eager("std::text::remove", remove, 3),
    FunctionRegistration::eager("std::text::replace", replace, 3),
    FunctionRegistration::eager("std::text::replace_first", replace_first, 3),
    FunctionRegistration::eager("std::text::replace_last", replace_last, 3),
    FunctionRegistration::eager("std::text::hex", hex, 1),
    FunctionRegistration::eager("std::text::octal", octal, 1),
    FunctionRegistration::eager("std::text::index_of", index_of, 2),
    FunctionRegistration::eager("std::text::contains", contains, 2),
    FunctionRegistration::eager("std::text::split", split, 2),
    FunctionRegistration::eager("std::text::starts_with", starts_with, 2),
    FunctionRegistration::eager("std::text::ends_with", ends_with, 2),
    FunctionRegistration::eager("std::text::to_ascii", to_ascii, 1),
    FunctionRegistration::eager("std::text::from_ascii", from_ascii, 1),
    FunctionRegistration::eager("std::text::encode", encode, 2),
    FunctionRegistration::eager("std::text::decode", decode, 2),
    FunctionRegistration::eager("std::text::is_equal", is_equal, 2),
];

fn arg_err<S: Into<String>>(msg: S) -> Signal {
    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "InvalidArgumentRuntimeError",
        msg.into(),
    ))
}

fn as_bytes(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let bytes: Vec<Value> = value
        .as_bytes()
        .iter()
        .map(|b| value_from_i64(*b as i64))
        .collect();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: bytes })),
    })
}

fn byte_size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);
    Signal::Success(value_from_i64(value.len() as i64))
}

fn capitalize(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let capitalized = value
        .split(' ')
        .map(|word| {
            if word.is_empty() {
                return String::from(word);
            }
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::from(word),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    Signal::Success(Value {
        kind: Some(Kind::StringValue(capitalized)),
    })
}

fn uppercase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_uppercase())),
    })
}

fn lowercase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_lowercase())),
    })
}

fn swapcase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let swapped = value
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().collect::<String>()
            } else if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(swapped)),
    })
}

fn trim(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.trim().to_string())),
    })
}

fn chars(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let list = value
        .chars()
        .map(|c| Value {
            kind: Some(Kind::StringValue(c.to_string())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: list })),
    })
}

fn at(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, index: tucana::shared::NumberValue);
    let index = match number_to_i64_lossy(&index) {
        Some(v) => v,
        None => return arg_err("Expected a number index"),
    };

    if index < 0 {
        return arg_err("Expected a non-negative index");
    }

    let idx = index as usize;
    match value.chars().nth(idx) {
        Some(c) => Signal::Success(Value {
            kind: Some(Kind::StringValue(c.to_string())),
        }),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!(
                "Index {} is out of bounds for string of length {}",
                index,
                value.chars().count()
            ),
        )),
    }
}

fn append(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, suffix: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value + &suffix)),
    })
}

fn prepend(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, prefix: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(prefix + &value)),
    })
}

fn insert(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, position: tucana::shared::NumberValue, text: String);
    let position = match number_to_i64_lossy(&position) {
        Some(v) => v,
        None => return arg_err("Expected a number position"),
    };

    if position < 0 {
        return arg_err("Expected a non-negative position");
    }

    let pos = position as usize;
    // Byte-wise position is kept intentionally to match existing flow behavior.
    if pos > value.len() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!("Position {} exceeds byte length {}", pos, value.len()),
        ));
    }

    let mut new_value = value;
    new_value.insert_str(pos, &text);

    Signal::Success(Value {
        kind: Some(Kind::StringValue(new_value)),
    })
}

fn length(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);
    Signal::Success(value_from_i64(value.chars().count() as i64))
}

fn remove(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, from: tucana::shared::NumberValue, to: tucana::shared::NumberValue);
    let from = match number_to_i64_lossy(&from) {
        Some(v) => v,
        None => return arg_err("Expected number 'from'"),
    };
    let to = match number_to_i64_lossy(&to) {
        Some(v) => v,
        None => return arg_err("Expected number 'to'"),
    };

    if from < 0 || to < 0 {
        return arg_err("Expected non-negative indices");
    }

    let from_u = from as usize;
    let to_u = to as usize;

    let chars = value.chars().collect::<Vec<char>>();
    if from_u > chars.len() || to_u > chars.len() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!(
                "Indices [{}, {}) out of bounds for length {}",
                from_u,
                to_u,
                chars.len()
            ),
        ));
    }

    let new = chars
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i < from_u || i >= to_u)
        .map(|e| e.1)
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(new)),
    })
}

fn replace(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, old: String, new: String);
    let replaced = value.replace(&old, &new);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn replace_first(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, old: String, new: String);
    let replaced = value.replacen(&old, &new, 1);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn replace_last(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, old: String, new: String);

    fn replace_last_impl(haystack: &str, needle: &str, replacement: &str) -> String {
        if let Some(pos) = haystack.rfind(needle) {
            let mut result =
                String::with_capacity(haystack.len() - needle.len() + replacement.len());
            result.push_str(&haystack[..pos]);
            result.push_str(replacement);
            result.push_str(&haystack[pos + needle.len()..]);
            result
        } else {
            haystack.to_string()
        }
    }

    let replaced = replace_last_impl(&value, &old, &new);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn hex(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let hex = value
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(hex)),
    })
}

fn octal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let oct = value
        .as_bytes()
        .iter()
        .map(|b| format!("{:03o}", b))
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(oct)),
    })
}

fn index_of(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, sub: String);

    match value.find(&sub) {
        Some(idx) => Signal::Success(value_from_i64(idx as i64)),
        None => Signal::Success(value_from_i64(-1)),
    }
}

fn contains(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, sub: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.contains(&sub))),
    })
}

fn split(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, delimiter: String);

    let parts = value
        .split(&delimiter)
        .map(|s| Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: parts })),
    })
}

fn reverse(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let reversed = value.chars().rev().collect::<String>();
    Signal::Success(Value {
        kind: Some(Kind::StringValue(reversed)),
    })
}

fn starts_with(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, prefix: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.starts_with(&prefix))),
    })
}

fn ends_with(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, suffix: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.ends_with(&suffix))),
    })
}

fn to_ascii(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String);

    let ascii = value
        .bytes()
        .map(|b| value_from_i64(b as i64))
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: ascii })),
    })
}

fn from_ascii(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    // Requires a TryFromArg impl for ListValue in your macro system.
    args!(args => list: ListValue);

    let string = list
        .values
        .iter()
        .map(|v| match v {
            Value {
                kind: Some(Kind::NumberValue(n)),
            } => match number_to_f64(n) {
                Some(n) if (0.0..=127.0).contains(&n) => Some(n as u8 as char),
                _ => None,
            },
            _ => None,
        })
        .collect::<Option<String>>();

    match string {
        Some(s) => Signal::Success(Value {
            kind: Some(Kind::StringValue(s)),
        }),
        None => arg_err("Expected a list of numbers between 0 and 127"),
    }
}

// NOTE: "encode"/"decode" currently only support base64.
fn encode(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, encoding: String);

    let encoded = match encoding.to_lowercase().as_str() {
        "base64" => base64::prelude::BASE64_STANDARD.encode(value),
        _ => {
            return arg_err(format!("Unsupported encoding: {}", encoding));
        }
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(encoded)),
    })
}

fn decode(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: String, encoding: String);

    let decoded = match encoding.to_lowercase().as_str() {
        "base64" => match base64::prelude::BASE64_STANDARD.decode(value) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(err) => {
                    return Signal::Failure(RuntimeError::new(
                        "T-STD-00001",
                        "DecodeError",
                        format!("Failed to decode base64 bytes to UTF-8: {:?}", err),
                    ));
                }
            },
            Err(err) => {
                return Signal::Failure(RuntimeError::new(
                    "T-STD-00001",
                    "DecodeError",
                    format!("Failed to decode base64 string: {:?}", err),
                ));
            }
        },
        _ => return arg_err(format!("Unsupported decoding: {}", encoding)),
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(decoded)),
    })
}

fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: String, rhs: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, value_from_f64, value_from_i64};
    use tucana::shared::{ListValue, Value, value::Kind};

    // ---------- helpers: build Arguments ----------
    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }
    fn a_num(n: f64) -> Argument {
        Argument::Eval(value_from_f64(n))
    }
    fn a_list(vals: Vec<Value>) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::ListValue(ListValue { values: vals })),
        })
    }

    // ---------- helpers: build bare Values ----------
    fn v_str(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }
    fn v_num(n: i64) -> Value {
        value_from_i64(n)
    }

    // ---------- helpers: extract from Signal ----------
    fn expect_num(sig: Signal) -> f64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_f64(&n).unwrap_or_default(),
            other => panic!("Expected NumberValue, got {:?}", other),
        }
    }
    fn expect_bool(sig: Signal) -> bool {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::BoolValue(b)),
            }) => b,
            other => panic!("Expected BoolValue, got {:?}", other),
        }
    }
    fn expect_str(sig: Signal) -> String {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::StringValue(s)),
            }) => s,
            other => panic!("Expected StringValue, got {:?}", other),
        }
    }
    fn expect_list(sig: Signal) -> Vec<Value> {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::ListValue(ListValue { values })),
            }) => values,
            other => panic!("Expected ListValue, got {:?}", other),
        }
    }

    // dummy runner for handlers that accept `run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal`
    fn dummy_run(_: i64, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    // ---------- tests ----------

    #[test]
    fn test_as_bytes_and_byte_size() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        // "hello" -> 5 bytes
        let bytes = expect_list(as_bytes(&[a_str("hello")], &mut ctx, &mut run));
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], v_num(104)); // 'h'

        let mut run = dummy_run;
        assert_eq!(
            expect_num(byte_size(&[a_str("hello")], &mut ctx, &mut run)),
            5.0
        );

        // unicode: "café" -> 5 bytes, 4 chars
        let mut run = dummy_run;
        assert_eq!(
            expect_num(byte_size(&[a_str("café")], &mut ctx, &mut run)),
            5.0
        );
        let mut run = dummy_run;
        assert_eq!(
            expect_num(length(&[a_str("café")], &mut ctx, &mut run)),
            4.0
        );
    }

    #[test]
    fn test_case_ops_and_trim() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(capitalize(&[a_str("hello world")], &mut ctx, &mut run)),
            "Hello World"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(uppercase(&[a_str("Hello")], &mut ctx, &mut run)),
            "HELLO"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(lowercase(&[a_str("Hello")], &mut ctx, &mut run)),
            "hello"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(swapcase(&[a_str("HeLLo123")], &mut ctx, &mut run)),
            "hEllO123"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(trim(&[a_str("  hi  ")], &mut ctx, &mut run)),
            "hi"
        );
    }

    #[test]
    fn test_chars_and_at() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let chars_list = expect_list(chars(&[a_str("abc")], &mut ctx, &mut run));
        assert_eq!(chars_list, vec![v_str("a"), v_str("b"), v_str("c")]);

        let mut run = dummy_run;
        assert_eq!(
            expect_str(at(&[a_str("hello"), a_num(1.0)], &mut ctx, &mut run)),
            "e"
        );

        // out-of-bounds
        let mut run = dummy_run;
        match at(&[a_str("hi"), a_num(5.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        // negative
        let mut run = dummy_run;
        match at(&[a_str("hi"), a_num(-1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
    }

    #[test]
    fn test_append_prepend_insert_length() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(append(
                &[a_str("hello"), a_str(" world")],
                &mut ctx,
                &mut run
            )),
            "hello world"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(prepend(
                &[a_str("world"), a_str("hello ")],
                &mut ctx,
                &mut run
            )),
            "hello world"
        );

        // insert uses BYTE index; for ASCII this matches char index
        let mut run = dummy_run;
        assert_eq!(
            expect_str(insert(
                &[a_str("hello"), a_num(2.0), a_str("XXX")],
                &mut ctx,
                &mut run
            )),
            "heXXXllo"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(length(&[a_str("hello")], &mut ctx, &mut run)),
            5.0
        );
    }

    #[test]
    fn test_remove_replace_variants() {
        let mut ctx = ValueStore::default();

        // remove uses CHAR indices [from, to)
        let mut run = dummy_run;
        assert_eq!(
            expect_str(remove(
                &[a_str("hello world"), a_num(2.0), a_num(7.0)],
                &mut ctx,
                &mut run
            )),
            "heorld"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace(
                &[a_str("hello world hello"), a_str("hello"), a_str("hi")],
                &mut ctx,
                &mut run
            )),
            "hi world hi"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace_first(
                &[a_str("one two one"), a_str("one"), a_str("1")],
                &mut ctx,
                &mut run
            )),
            "1 two one"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace_last(
                &[a_str("one two one"), a_str("one"), a_str("1")],
                &mut ctx,
                &mut run
            )),
            "one two 1"
        );
    }

    #[test]
    fn test_hex_octal_reverse() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(hex(&[a_str("hello")], &mut ctx, &mut run)),
            "68656c6c6f"
        );

        let mut run = dummy_run;
        assert_eq!(expect_str(octal(&[a_str("A")], &mut ctx, &mut run)), "101");

        let mut run = dummy_run;
        assert_eq!(
            expect_str(reverse(&[a_str("hello")], &mut ctx, &mut run)),
            "olleh"
        );
    }

    #[test]
    fn test_index_contains_split_starts_ends() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(index_of(
                &[a_str("hello world"), a_str("world")],
                &mut ctx,
                &mut run
            )),
            6.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(index_of(
                &[a_str("hello"), a_str("xyz")],
                &mut ctx,
                &mut run
            )),
            -1.0
        );

        let mut run = dummy_run;
        assert!(expect_bool(contains(
            &[a_str("hello world"), a_str("world")],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        let split_list = expect_list(split(&[a_str("a,b,c"), a_str(",")], &mut ctx, &mut run));
        assert_eq!(split_list, vec![v_str("a"), v_str("b"), v_str("c")]);

        let mut run = dummy_run;
        assert!(expect_bool(starts_with(
            &[a_str("hello"), a_str("he")],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        assert!(expect_bool(ends_with(
            &[a_str("hello"), a_str("lo")],
            &mut ctx,
            &mut run
        )));
    }

    #[test]
    fn test_to_ascii_and_from_ascii() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let ascii_vals = expect_list(to_ascii(&[a_str("AB")], &mut ctx, &mut run));
        assert_eq!(ascii_vals, vec![v_num(65), v_num(66)]);

        let mut run = dummy_run;
        let list_arg = a_list(vec![v_num(65), v_num(66), v_num(67)]);
        assert_eq!(
            expect_str(from_ascii(&[list_arg], &mut ctx, &mut run)),
            "ABC"
        );

        // invalid element
        let mut run = dummy_run;
        let list_arg = a_list(vec![v_num(65), v_num(128)]);
        match from_ascii(&[list_arg], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for invalid ASCII, got {:?}", s),
        }
    }

    #[test]
    fn test_encode_decode_base64_and_is_equal() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(encode(
                &[a_str("hello"), a_str("BASE64")],
                &mut ctx,
                &mut run
            )),
            "aGVsbG8="
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(decode(
                &[a_str("aGVsbG8="), a_str("base64")],
                &mut ctx,
                &mut run
            )),
            "hello"
        );

        // unsupported codec
        let mut run = dummy_run;
        match encode(&[a_str("data"), a_str("gug")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for unsupported encoding, got {:?}", s),
        }

        let mut run = dummy_run;
        assert!(expect_bool(is_equal(
            &[a_str("x"), a_str("x")],
            &mut ctx,
            &mut run
        )));
        let mut run = dummy_run;
        assert!(!expect_bool(is_equal(
            &[a_str("x"), a_str("y")],
            &mut ctx,
            &mut run
        )));
    }
}
