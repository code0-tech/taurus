use tucana::shared::{value::Kind, ListValue, Value};

use crate::{context::Context, error::RuntimeError};

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

            let upper = String::from(&word[..0]).to_uppercase();
            return String::from(upper + &word[1..]);
        })
        .collect();

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
