use crate::{context::Context, error::RuntimeError, registry::HandlerFn};
use tucana::shared::{value::Kind, ListValue, Value};

pub fn collect_text_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::text::as_bytes", as_bytes),
        ("std::text::byte_size", byte_size),
        ("std::text::capitalize", capitalize),
        ("std::text::lowercase", lowercase),
        ("std::text::uppercase", uppercase),
        ("std::text::swapcase", swapcase),
        ("std::text::trim", trim),
        ("std::text::chars", chars),
        ("std::text::at", at),
        ("std::text::append", append),
        ("std::text::prepend", prepend),
        ("std::text::insert", insert),
        ("std::text::length", length),
        ("std::text::reverse", reverse),
        ("std::text::remove", remove),
        ("std::text::replace", replace),
        ("std::text::replace_first", replace_first),
        ("std::text::replace_last", replace_last),
        ("std::text::hex", hex),
        ("std::text::octal", octal),
        ("std::text::index_of", index_of),
        ("std::text::contains", contains),
        ("std::text::split", split),
        ("std::text::starts_with", starts_with),
        ("std::text::ends_with", ends_with),
        ("std::text::to_ascii", to_ascii),
        ("std::text::from_ascii", from_ascii),
        ("std::text::encode", encode),
        ("std::text::decode", decode),
        ("std::text::is_equal", is_equal),
    ]
}

fn as_bytes(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let bytes: Vec<Value> = value
        .as_bytes()
        .iter()
        .map(|byte| Value {
            kind: Some(Kind::NumberValue(*byte as f64)),
        })
        .collect();

    Ok(Value {
        kind: Some(Kind::ListValue(ListValue { values: bytes })),
    })
}

fn byte_size(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.len() as f64)),
    })
}

fn capitalize(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let capitalized_value = value
        .split(" ")
        .map(|word| {
            if word.is_empty() {
                return String::from(word);
            }

            if word.len() == 1 {
                return String::from(word.to_uppercase());
            }

            let first = word.chars().nth(0);

            if first.is_some() {
                let first = first.unwrap();
                String::from(first).to_uppercase() + &word[1..]
            } else {
                String::from(word)
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    Ok(Value {
        kind: Some(Kind::StringValue(capitalized_value)),
    })
}

fn uppercase(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_uppercase())),
    })
}

fn lowercase(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_lowercase())),
    })
}

fn swapcase(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

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
        .collect();

    Ok(Value {
        kind: Some(Kind::StringValue(swapped)),
    })
}

fn chars(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let chars = value
        .chars()
        .map(|c| Value {
            kind: Some(Kind::StringValue(c.to_string())),
        })
        .collect::<Vec<Value>>();

    Ok(Value {
        kind: Some(Kind::ListValue(ListValue { values: chars })),
    })
}

fn at(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::NumberValue(index)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    if index < &0.0 {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected a positive number as the second argument but received {}",
                index
            ),
        ));
    }

    let usize_index = index.clone() as usize;
    let char = value.chars().into_iter().nth(usize_index);

    match char {
        Some(c) => Ok(Value {
            kind: Some(Kind::StringValue(c.to_string())),
        }),
        None => Err(RuntimeError::simple(
            "IndexOutOfBoundsRuntimeError",
            format!(
                "Index {} is out of bounds for string of length {}",
                index,
                value.len()
            ),
        )),
    }
}

fn trim(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.trim().to_string())),
    })
}

fn append(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(suffix)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.clone() + suffix)),
    })
}

fn prepend(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(prefix)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(prefix.clone() + value)),
    })
}

fn insert(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::NumberValue(position)),
    }, Value {
        kind: Some(Kind::StringValue(text)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let usize_position = position.clone() as usize;
    let mut new_value = value.clone();
    new_value.insert_str(usize_position, text.as_str());

    Ok(Value {
        kind: Some(Kind::StringValue(new_value)),
    })
}

fn length(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.chars().count() as f64)),
    })
}

fn remove(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::NumberValue(from)),
    }, Value {
        kind: Some(Kind::NumberValue(to)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let chars = value.chars().into_iter().collect::<Vec<char>>();

    let new = chars
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i < from.clone() as usize || i >= to.clone() as usize)
        .map(|e| e.1)
        .collect::<String>();

    Ok(Value {
        kind: Some(Kind::StringValue(new)),
    })
}

fn replace(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(old)),
    }, Value {
        kind: Some(Kind::StringValue(new)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let replaced = value.replace(old, new);

    Ok(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn replace_first(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(old)),
    }, Value {
        kind: Some(Kind::StringValue(new)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let replaced = value.replacen(old, new, 1);

    Ok(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn replace_last(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(old)),
    }, Value {
        kind: Some(Kind::StringValue(new)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    fn replace_last(haystack: &str, needle: &str, replacement: &str) -> String {
        if let Some(pos) = haystack.rfind(needle) {
            let mut result =
                String::with_capacity(haystack.len() - needle.len() + replacement.len());
            result.push_str(&haystack[..pos]);
            result.push_str(replacement);
            result.push_str(&haystack[pos + needle.len()..]);
            result
        } else {
            haystack.to_string() // kein Vorkommen gefunden, original zurückgeben
        }
    }

    let replaced = replace_last(value.as_str(), old.as_str(), new.as_str());

    Ok(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

fn hex(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let hex = value
        .as_bytes()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    Ok(Value {
        kind: Some(Kind::StringValue(hex)),
    })
}

fn octal(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let hex = value
        .as_bytes()
        .iter()
        .map(|byte| format!("{:03o}", byte))
        .collect::<String>();

    Ok(Value {
        kind: Some(Kind::StringValue(hex)),
    })
}

fn index_of(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(sub_string)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let index_option = value.find(sub_string);

    match index_option {
        Some(index) => Ok(Value {
            kind: Some(Kind::NumberValue(index as f64)),
        }),
        None => Ok(Value {
            kind: Some(Kind::NumberValue(-1.0)),
        }),
    }
}

fn contains(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(sub_string)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value.contains(sub_string))),
    })
}

fn split(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(delimiter)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let words = value
        .split(delimiter)
        .map(|word| Value {
            kind: Some(Kind::StringValue(word.to_string())),
        })
        .collect::<Vec<Value>>();

    Ok(Value {
        kind: Some(Kind::ListValue(ListValue { values: words })),
    })
}

fn reverse(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let reversed = value.chars().rev().collect::<String>();

    Ok(Value {
        kind: Some(Kind::StringValue(reversed)),
    })
}

fn starts_with(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(prefix)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value.starts_with(prefix))),
    })
}

fn ends_with(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }, Value {
        kind: Some(Kind::StringValue(suffix)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value.ends_with(suffix))),
    })
}

fn to_ascii(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(value)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let ascii_value = value
        .bytes()
        .map(|b| Value {
            kind: Some(Kind::NumberValue(b as f64)),
        })
        .collect::<Vec<Value>>();

    Ok(Value {
        kind: Some(Kind::ListValue(ListValue {
            values: ascii_value,
        })),
    })
}

// todo!("from_ascii")

fn from_ascii(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::ListValue(list)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let string = list
        .values
        .iter()
        .map(|number_value| {
            if let Value {
                kind: Some(Kind::NumberValue(number)),
            } = number_value
            {
                if number >= &0.0 && number <= &127.0 {
                    Some(number.clone() as u8 as char)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Option<String>>();

    match string {
        Some(string) => Ok(Value {
            kind: Some(Kind::StringValue(string)),
        }),
        None => Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Expected a list of numbers between 0 and 127".to_string(),
        )),
    }
}

//TODO: Implement encode function , what about decode? UTF-8, 16 and 32 does not make sense

fn encode(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    todo!("not implemented")
}

fn decode(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    todo!("not implemented")
}

fn is_equal(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(lhs)),
    }, Value {
        kind: Some(Kind::StringValue(rhs)),
    }] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use tucana::shared::{value::Kind, ListValue, Value};

    // Helper function to create a string value
    fn create_string_value(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }

    // Helper function to create a number value
    fn create_number_value(num: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(num)),
        }
    }

    // Helper function to create a bool value
    fn create_bool_value(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }

    // Helper function to create a list value
    fn create_list_value(values: Vec<Value>) -> Value {
        Value {
            kind: Some(Kind::ListValue(ListValue { values })),
        }
    }

    #[test]
    fn test_as_bytes_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = as_bytes(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 5);
            assert_eq!(list.values[0], create_number_value(104.0)); // 'h'
            assert_eq!(list.values[1], create_number_value(101.0)); // 'e'
            assert_eq!(list.values[2], create_number_value(108.0)); // 'l'
            assert_eq!(list.values[3], create_number_value(108.0)); // 'l'
            assert_eq!(list.values[4], create_number_value(111.0)); // 'o'
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_as_bytes_empty_string() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = as_bytes(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 0);
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_as_bytes_invalid_argument() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(123.0)];
        let result = as_bytes(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_byte_size_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = byte_size(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(5.0));
    }

    #[test]
    fn test_byte_size_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = byte_size(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(0.0));
    }

    #[test]
    fn test_byte_size_unicode() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("café")];
        let result = byte_size(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(5.0)); // 'é' is 2 bytes in UTF-8
    }

    #[test]
    fn test_capitalize_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello world")];
        let result = capitalize(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("Hello World"));
    }

    #[test]
    fn test_capitalize_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = capitalize(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value(""));
    }

    #[test]
    fn test_capitalize_single_char() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("a")];
        let result = capitalize(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("A"));
    }

    #[test]
    fn test_uppercase_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("Hello World")];
        let result = uppercase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("HELLO WORLD"));
    }

    #[test]
    fn test_uppercase_already_upper() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("HELLO")];
        let result = uppercase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("HELLO"));
    }

    #[test]
    fn test_lowercase_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("Hello World")];
        let result = lowercase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_lowercase_already_lower() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = lowercase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello"));
    }

    #[test]
    fn test_swapcase_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("Hello World")];
        let result = swapcase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hELLO wORLD"));
    }

    #[test]
    fn test_swapcase_mixed() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("HeLLo123")];
        let result = swapcase(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hEllO123"));
    }

    #[test]
    fn test_chars_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("abc")];
        let result = chars(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 3);
            assert_eq!(list.values[0], create_string_value("a"));
            assert_eq!(list.values[1], create_string_value("b"));
            assert_eq!(list.values[2], create_string_value("c"));
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_chars_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = chars(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 0);
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_at_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_number_value(1.0)];
        let result = at(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("e"));
    }

    #[test]
    fn test_at_first_char() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_number_value(0.0)];
        let result = at(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("h"));
    }

    #[test]
    fn test_at_out_of_bounds() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_number_value(10.0)];
        let result = at(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_at_negative_index() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_number_value(-1.0)];
        let result = at(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_trim_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("  hello world  ")];
        let result = trim(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_trim_no_whitespace() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = trim(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello"));
    }

    #[test]
    fn test_append_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value(" world")];
        let result = append(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_append_empty_suffix() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value("")];
        let result = append(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello"));
    }

    #[test]
    fn test_prepend_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("world"), create_string_value("hello ")];
        let result = prepend(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_prepend_empty_prefix() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value("")];
        let result = prepend(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello"));
    }

    #[test]
    fn test_insert_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello"),
            create_number_value(2.0),
            create_string_value("XXX"),
        ];
        let result = insert(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("heXXXllo"));
    }

    #[test]
    fn test_insert_at_beginning() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello"),
            create_number_value(0.0),
            create_string_value("XXX"),
        ];
        let result = insert(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("XXXhello"));
    }

    #[test]
    fn test_insert_at_end() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello"),
            create_number_value(5.0),
            create_string_value("XXX"),
        ];
        let result = insert(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("helloXXX"));
    }

    #[test]
    fn test_length_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = length(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(5.0));
    }

    #[test]
    fn test_length_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = length(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(0.0));
    }

    #[test]
    fn test_length_unicode() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("café")];
        let result = length(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(4.0)); // 4 characters
    }

    #[test]
    fn test_remove_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_number_value(2.0),
            create_number_value(7.0),
        ];
        let result = remove(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("heorld"));
    }

    #[test]
    fn test_remove_from_beginning() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello"),
            create_number_value(0.0),
            create_number_value(2.0),
        ];
        let result = remove(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("llo"));
    }

    #[test]
    fn test_replace_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world hello"),
            create_string_value("hello"),
            create_string_value("hi"),
        ];
        let result = replace(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hi world hi"));
    }

    #[test]
    fn test_replace_not_found() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("xyz"),
            create_string_value("abc"),
        ];
        let result = replace(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_replace_first_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world hello"),
            create_string_value("hello"),
            create_string_value("hi"),
        ];
        let result = replace_first(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hi world hello"));
    }

    #[test]
    fn test_replace_last_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world hello"),
            create_string_value("hello"),
            create_string_value("hi"),
        ];
        let result = replace_last(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world hi"));
    }

    #[test]
    fn test_replace_last_not_found() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("xyz"),
            create_string_value("abc"),
        ];
        let result = replace_last(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello world"));
    }

    #[test]
    fn test_hex_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = hex(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("68656c6c6f"));
    }

    #[test]
    fn test_hex_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = hex(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value(""));
    }

    #[test]
    fn test_octal_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("A")];
        let result = octal(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("101")); // 'A' is 65 in ASCII, 101 in octal
    }

    #[test]
    fn test_index_of_found() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("world"),
        ];
        let result = index_of(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(6.0));
    }

    #[test]
    fn test_index_of_not_found() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("xyz"),
        ];
        let result = index_of(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(-1.0));
    }

    #[test]
    fn test_index_of_at_beginning() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("hello"),
        ];
        let result = index_of(&values, &mut ctx).unwrap();
        assert_eq!(result, create_number_value(0.0));
    }

    #[test]
    fn test_contains_true() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("world"),
        ];
        let result = contains(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(true));
    }

    #[test]
    fn test_contains_false() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("xyz"),
        ];
        let result = contains(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(false));
    }

    #[test]
    fn test_split_valid() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello,world,test"),
            create_string_value(","),
        ];
        let result = split(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 3);
            assert_eq!(list.values[0], create_string_value("hello"));
            assert_eq!(list.values[1], create_string_value("world"));
            assert_eq!(list.values[2], create_string_value("test"));
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_split_no_delimiter() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value(",")];
        let result = split(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 1);
            assert_eq!(list.values[0], create_string_value("hello"));
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_reverse_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello")];
        let result = reverse(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("olleh"));
    }

    #[test]
    fn test_reverse_empty() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("")];
        let result = reverse(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value(""));
    }

    #[test]
    fn test_reverse_palindrome() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("aba")];
        let result = reverse(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("aba"));
    }

    #[test]
    fn test_starts_with_true() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("hello"),
        ];
        let result = starts_with(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(true));
    }

    #[test]
    fn test_starts_with_false() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("world"),
        ];
        let result = starts_with(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(false));
    }

    #[test]
    fn test_ends_with_true() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("world"),
        ];
        let result = ends_with(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(true));
    }

    #[test]
    fn test_ends_with_false() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("hello world"),
            create_string_value("hello"),
        ];
        let result = ends_with(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(false));
    }

    #[test]
    fn test_to_ascii_valid() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("AB")];
        let result = to_ascii(&values, &mut ctx).unwrap();

        if let Value {
            kind: Some(Kind::ListValue(list)),
        } = result
        {
            assert_eq!(list.values.len(), 2);
            assert_eq!(list.values[0], create_number_value(65.0)); // 'A'
            assert_eq!(list.values[1], create_number_value(66.0)); // 'B'
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_from_ascii_valid() {
        let mut ctx = Context::new();
        let ascii_values = vec![
            create_number_value(65.0), // 'A'
            create_number_value(66.0), // 'B'
            create_number_value(67.0), // 'C'
        ];
        let values = vec![create_list_value(ascii_values)];
        let result = from_ascii(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("ABC"));
    }

    #[test]
    fn test_from_ascii_empty_list() {
        let mut ctx = Context::new();
        let values = vec![create_list_value(vec![])];
        let result = from_ascii(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value(""));
    }

    #[test]
    fn test_from_ascii_invalid_range() {
        let mut ctx = Context::new();
        let ascii_values = vec![
            create_number_value(65.0),
            create_number_value(128.0), // Out of ASCII range
        ];
        let values = vec![create_list_value(ascii_values)];
        let result = from_ascii(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_ascii_negative_number() {
        let mut ctx = Context::new();
        let ascii_values = vec![
            create_number_value(65.0),
            create_number_value(-1.0), // Negative
        ];
        let values = vec![create_list_value(ascii_values)];
        let result = from_ascii(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_ascii_non_number_in_list() {
        let mut ctx = Context::new();
        let ascii_values = vec![
            create_number_value(65.0),
            create_string_value("invalid"), // Non-number
        ];
        let values = vec![create_list_value(ascii_values)];
        let result = from_ascii(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_equal_true() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value("hello")];
        let result = is_equal(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(true));
    }

    #[test]
    fn test_is_equal_false() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("hello"), create_string_value("world")];
        let result = is_equal(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(false));
    }

    #[test]
    fn test_is_equal_empty_strings() {
        let mut ctx = Context::new();
        let values = vec![create_string_value(""), create_string_value("")];
        let result = is_equal(&values, &mut ctx).unwrap();
        assert_eq!(result, create_bool_value(true));
    }

    // Test invalid arguments for functions requiring specific argument counts
    #[test]
    fn test_invalid_arguments_single_string() {
        let mut ctx = Context::new();

        // Test functions that expect 1 string argument
        let invalid_values = vec![create_number_value(123.0)];

        assert!(as_bytes(&invalid_values, &mut ctx).is_err());
        assert!(byte_size(&invalid_values, &mut ctx).is_err());
        assert!(capitalize(&invalid_values, &mut ctx).is_err());
        assert!(uppercase(&invalid_values, &mut ctx).is_err());
        assert!(lowercase(&invalid_values, &mut ctx).is_err());
        assert!(swapcase(&invalid_values, &mut ctx).is_err());
        assert!(chars(&invalid_values, &mut ctx).is_err());
        assert!(trim(&invalid_values, &mut ctx).is_err());
        assert!(length(&invalid_values, &mut ctx).is_err());
        assert!(reverse(&invalid_values, &mut ctx).is_err());
        assert!(hex(&invalid_values, &mut ctx).is_err());
        assert!(octal(&invalid_values, &mut ctx).is_err());
        assert!(to_ascii(&invalid_values, &mut ctx).is_err());
    }

    #[test]
    fn test_invalid_arguments_two_strings() {
        let mut ctx = Context::new();

        // Test functions that expect 2 string arguments
        let invalid_values = vec![create_string_value("hello"), create_number_value(123.0)];

        assert!(append(&invalid_values, &mut ctx).is_err());
        assert!(prepend(&invalid_values, &mut ctx).is_err());
        assert!(index_of(&invalid_values, &mut ctx).is_err());
        assert!(contains(&invalid_values, &mut ctx).is_err());
        assert!(split(&invalid_values, &mut ctx).is_err());
        assert!(starts_with(&invalid_values, &mut ctx).is_err());
        assert!(ends_with(&invalid_values, &mut ctx).is_err());
        assert!(is_equal(&invalid_values, &mut ctx).is_err());
    }

    #[test]
    fn test_invalid_arguments_string_and_number() {
        let mut ctx = Context::new();

        // Test functions that expect string and number
        let invalid_values = vec![create_number_value(123.0), create_string_value("test")];

        assert!(at(&invalid_values, &mut ctx).is_err());
    }

    #[test]
    fn test_invalid_arguments_three_params() {
        let mut ctx = Context::new();

        // Test functions that expect 3 arguments
        let invalid_values = vec![
            create_string_value("hello"),
            create_string_value("test"),
            create_number_value(123.0),
        ];

        assert!(insert(&invalid_values, &mut ctx).is_err());
        assert!(remove(&invalid_values, &mut ctx).is_err());
        assert!(replace(&invalid_values, &mut ctx).is_err());
        assert!(replace_first(&invalid_values, &mut ctx).is_err());
        assert!(replace_last(&invalid_values, &mut ctx).is_err());
    }

    #[test]
    fn test_wrong_argument_count() {
        let mut ctx = Context::new();

        // Test with wrong number of arguments
        let empty_values = vec![];
        let too_many_values = vec![
            create_string_value("test1"),
            create_string_value("test2"),
            create_string_value("test3"),
            create_string_value("test4"),
        ];

        assert!(as_bytes(&empty_values, &mut ctx).is_err());
        assert!(as_bytes(&too_many_values, &mut ctx).is_err());

        assert!(append(&empty_values, &mut ctx).is_err());
        assert!(append(&too_many_values, &mut ctx).is_err());
    }

    #[test]
    fn test_edge_cases() {
        let mut ctx = Context::new();

        // Test with very long string
        let long_string = "a".repeat(1000);
        let values = vec![create_string_value(&long_string)];

        assert!(length(&values, &mut ctx).is_ok());
        assert!(reverse(&values, &mut ctx).is_ok());
        assert!(uppercase(&values, &mut ctx).is_ok());

        // Test with special characters
        let special_string = "!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";
        let values = vec![create_string_value(special_string)];

        assert!(length(&values, &mut ctx).is_ok());
        assert!(reverse(&values, &mut ctx).is_ok());
        assert!(uppercase(&values, &mut ctx).is_ok());

        // Test with unicode characters
        let unicode_string = "🦀🚀✨🎉";
        let values = vec![create_string_value(unicode_string)];

        assert!(length(&values, &mut ctx).is_ok());
        assert!(reverse(&values, &mut ctx).is_ok());
        assert!(chars(&values, &mut ctx).is_ok());
    }

    #[test]
    fn test_boundary_conditions() {
        let mut ctx = Context::new();

        // Test insert at various positions
        let base_string = "hello";
        for i in 0..=5 {
            let values = vec![
                create_string_value(base_string),
                create_number_value(i as f64),
                create_string_value("X"),
            ];
            let result = insert(&values, &mut ctx);
            assert!(result.is_ok(), "Insert at position {} should work", i);
        }

        // Test remove with edge cases
        let values = vec![
            create_string_value("hello"),
            create_number_value(0.0),
            create_number_value(0.0), // Remove nothing
        ];
        let result = remove(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value("hello"));

        // Test remove entire string
        let values = vec![
            create_string_value("hello"),
            create_number_value(0.0),
            create_number_value(5.0),
        ];
        let result = remove(&values, &mut ctx).unwrap();
        assert_eq!(result, create_string_value(""));
    }
}
