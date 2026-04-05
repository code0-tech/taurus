use tucana::shared::{NumberValue, Value, number_value, value::Kind};

pub fn number_value_from_f64(n: f64) -> NumberValue {
    NumberValue {
        number: Some(number_value::Number::Float(n)),
    }
}

pub fn number_value_from_i64(n: i64) -> NumberValue {
    NumberValue {
        number: Some(number_value::Number::Integer(n)),
    }
}

pub fn value_from_f64(n: f64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(number_value_from_f64(n))),
    }
}

pub fn value_from_i64(n: i64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(number_value_from_i64(n))),
    }
}

pub fn number_to_f64(n: &NumberValue) -> Option<f64> {
    match n.number {
        Some(number_value::Number::Integer(i)) => Some(i as f64),
        Some(number_value::Number::Float(f)) => Some(f),
        None => None,
    }
}

pub fn number_to_i64_lossy(n: &NumberValue) -> Option<i64> {
    match n.number {
        Some(number_value::Number::Integer(i)) => Some(i),
        Some(number_value::Number::Float(f)) => {
            if f.is_finite() {
                Some(f as i64)
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn number_to_string(n: &NumberValue) -> String {
    match n.number {
        Some(number_value::Number::Integer(i)) => i.to_string(),
        Some(number_value::Number::Float(f)) => f.to_string(),
        None => "null".to_string(),
    }
}
