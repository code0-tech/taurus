//! Numeric standard-library handlers.
//!
//! Most operators keep an integer fast-path (checked ops) and fall back to `f64` arithmetic
//! when needed so common integer-heavy flows avoid unnecessary float conversion.

use std::f64;

use tucana::shared::helper::value::ToValue;
use tucana::shared::{NumberValue, Value, number_value, value::Kind};

use crate::handler::argument::Argument;
use crate::handler::macros::{args, no_args};
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::{number_to_f64, number_to_i64_lossy, value_from_f64, value_from_i64};

fn num_f64(n: &NumberValue) -> Result<f64, Signal> {
    // Centralized conversion keeps all numeric argument failures consistent.
    number_to_f64(n).ok_or_else(|| {
        Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected number",
        ))
    })
}

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("std::number::add", add, 2),
    FunctionRegistration::eager("std::number::multiply", multiply, 2),
    FunctionRegistration::eager("std::number::substract", substract, 2),
    FunctionRegistration::eager("std::number::divide", divide, 2),
    FunctionRegistration::eager("std::number::modulo", modulo, 2),
    FunctionRegistration::eager("std::number::abs", abs, 1),
    FunctionRegistration::eager("std::number::is_positive", is_positive, 1),
    FunctionRegistration::eager("std::number::is_greater", is_greater, 2),
    FunctionRegistration::eager("std::number::is_less", is_less, 2),
    FunctionRegistration::eager("std::number::is_zero", is_zero, 1),
    FunctionRegistration::eager("std::number::square", square, 2),
    FunctionRegistration::eager("std::number::exponential", exponential, 2),
    FunctionRegistration::eager("std::number::pi", pi, 0),
    FunctionRegistration::eager("std::number::euler", euler, 0),
    FunctionRegistration::eager("std::number::infinity", infinity, 0),
    FunctionRegistration::eager("std::number::round_up", round_up, 2),
    FunctionRegistration::eager("std::number::round_down", round_down, 2),
    FunctionRegistration::eager("std::number::round", round, 2),
    FunctionRegistration::eager("std::number::square_root", square_root, 1),
    FunctionRegistration::eager("std::number::root", root, 2),
    FunctionRegistration::eager("std::number::log", log, 2),
    FunctionRegistration::eager("std::number::ln", ln, 1),
    FunctionRegistration::eager("std::number::from_text", from_text, 1),
    FunctionRegistration::eager("std::number::as_text", as_text, 1),
    FunctionRegistration::eager("std::number::min", min, 2),
    FunctionRegistration::eager("std::number::max", max, 2),
    FunctionRegistration::eager("std::number::negate", negate, 1),
    FunctionRegistration::eager("std::number::random_number", random, 2),
    FunctionRegistration::eager("std::number::sin", sin, 1),
    FunctionRegistration::eager("std::number::cos", cos, 1),
    FunctionRegistration::eager("std::number::tan", tan, 1),
    FunctionRegistration::eager("std::number::arcsin", arcsin, 1),
    FunctionRegistration::eager("std::number::arccos", arccos, 1),
    FunctionRegistration::eager("std::number::arctan", arctan, 1),
    FunctionRegistration::eager("std::number::sinh", sinh, 1),
    FunctionRegistration::eager("std::number::cosh", cosh, 1),
    FunctionRegistration::eager("std::number::clamp", clamp, 3),
    FunctionRegistration::eager("std::number::is_equal", is_equal, 2),
    FunctionRegistration::eager("std::number::has_digits", has_digits, 2),
    FunctionRegistration::eager("std::number::remove_digits", remove_digits, 2),
];

fn has_digits(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);

    match value.number {
        Some(number) => match number {
            number_value::Number::Integer(_) => Signal::Success(false.to_value()),
            number_value::Number::Float(_) => Signal::Success(true.to_value()),
        },
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvlaidArgumentExeption",
            "Had NumberValue but no inner number value (was null)",
        )),
    }
}

fn remove_digits(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    match number_to_i64_lossy(&value) {
        Some(number) => Signal::Success(value_from_i64(number)),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvlaidArgumentExeption",
            "Had NumberValue but no inner number value (was null)",
        )),
    }
}

fn add(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    // Preserve integer precision and overflow checks when both operands are integers.
    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
        && let Some(sum) = a.checked_add(b)
    {
        return Signal::Success(value_from_i64(sum));
    }
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs + rhs))
}

fn multiply(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
        && let Some(prod) = a.checked_mul(b)
    {
        return Signal::Success(value_from_i64(prod));
    }
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs * rhs))
}

fn substract(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
        && let Some(diff) = a.checked_sub(b)
    {
        return Signal::Success(value_from_i64(diff));
    }
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs - rhs))
}

fn divide(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);

    let rhs_f = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };

    if rhs_f == 0.0 {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
        && b != 0
        && a % b == 0
    {
        return Signal::Success(value_from_i64(a / b));
    }

    let lhs_f = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs_f / rhs_f))
}

fn modulo(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);

    let rhs_f = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };

    if rhs_f == 0.0 {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
        && b != 0
    {
        return Signal::Success(value_from_i64(a % b));
    }

    let lhs_f = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs_f % rhs_f))
}

fn abs(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    if let Some(number_value::Number::Integer(i)) = value.number
        && let Some(abs) = i.checked_abs()
    {
        return Signal::Success(value_from_i64(abs));
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.abs()))
}

fn is_positive(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!value.is_sign_negative())),
    })
}

fn is_greater(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs > rhs)),
    })
}

fn is_less(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs < rhs)),
    })
}

fn is_zero(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value == 0.0)),
    })
}

fn square(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    if let Some(number_value::Number::Integer(i)) = value.number
        && let Some(prod) = i.checked_mul(i)
    {
        return Signal::Success(value_from_i64(prod));
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.powf(2.0)))
}

fn exponential(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => base: NumberValue, exponent: NumberValue);
    match (base.number, exponent.number) {
        (Some(number_value::Number::Integer(b)), Some(number_value::Number::Integer(e)))
            if e >= 0 =>
        {
            if let Ok(exp) = u32::try_from(e)
                && let Some(pow) = b.checked_pow(exp)
            {
                return Signal::Success(value_from_i64(pow));
            }
        }
        _ => {}
    }
    let base = match num_f64(&base) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let exponent = match num_f64(&exponent) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(base.powf(exponent)))
}

fn pi(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_f64(f64::consts::PI))
}

fn euler(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_f64(f64::consts::E))
}

fn infinity(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_f64(f64::INFINITY))
}

fn round_up(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, decimal_places: NumberValue);
    let decimal_places = match num_f64(&decimal_places) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match value.number {
        Some(number_value::Number::Integer(i)) if decimal_places <= 0.0 => {
            return Signal::Success(value_from_i64(i));
        }
        _ => {}
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(value_from_f64((value * factor).ceil() / factor))
}

fn round_down(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, decimal_places: NumberValue);
    let decimal_places = match num_f64(&decimal_places) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match value.number {
        Some(number_value::Number::Integer(i)) if decimal_places <= 0.0 => {
            return Signal::Success(value_from_i64(i));
        }
        _ => {}
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(value_from_f64((value * factor).floor() / factor))
}

fn round(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, decimal_places: NumberValue);
    let decimal_places = match num_f64(&decimal_places) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match value.number {
        Some(number_value::Number::Integer(i)) if decimal_places <= 0.0 => {
            return Signal::Success(value_from_i64(i));
        }
        _ => {}
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(value_from_f64((value * factor).round() / factor))
}

fn square_root(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sqrt()))
}

fn root(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, root: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let root = match num_f64(&root) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.powf(root)))
}

fn log(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, base: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let base = match num_f64(&base) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.log(base)))
}

fn ln(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.ln()))
}

fn from_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => string_value: String);

    if let Ok(v) = string_value.parse::<i64>() {
        return Signal::Success(value_from_i64(v));
    }
    match string_value.parse::<f64>() {
        Ok(v) => Signal::Success(value_from_f64(v)),
        Err(_) => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Failed to parse string as number: {}", string_value),
        )),
    }
}

fn as_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn min(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
    {
        return Signal::Success(value_from_i64(a.min(b)));
    }
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs.min(rhs)))
}

fn max(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    if let (Some(number_value::Number::Integer(a)), Some(number_value::Number::Integer(b))) =
        (lhs.number, rhs.number)
    {
        return Signal::Success(value_from_i64(a.max(b)));
    }
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(lhs.max(rhs)))
}

fn negate(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    if let Some(number_value::Number::Integer(i)) = value.number
        && let Some(neg) = i.checked_neg()
    {
        return Signal::Success(value_from_i64(neg));
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(-value))
}

fn random(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => min: NumberValue, max: NumberValue);

    let min_f = match num_f64(&min) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let max_f = match num_f64(&max) {
        Ok(v) => v,
        Err(e) => return e,
    };

    if min_f > max_f {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidRange",
            "First number can't be bigger then second when creating a range for std::math::random",
        ));
    }

    let value = rand::random_range(min_f..=max_f);

    Signal::Success(value_from_f64(value))
}

fn sin(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sin()))
}

fn cos(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.cos()))
}

fn tan(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.tan()))
}

fn arcsin(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.asin()))
}

fn arccos(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.acos()))
}

fn arctan(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.atan()))
}

fn sinh(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sinh()))
}

fn cosh(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.cosh()))
}

fn clamp(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: NumberValue, min: NumberValue, max: NumberValue);
    if let (
        Some(number_value::Number::Integer(v)),
        Some(number_value::Number::Integer(min)),
        Some(number_value::Number::Integer(max)),
    ) = (value.number, min.number, max.number)
    {
        return Signal::Success(value_from_i64(v.clamp(min, max)));
    }
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let min = match num_f64(&min) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let max = match num_f64(&max) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.clamp(min, max)))
}

fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => lhs: NumberValue, rhs: NumberValue);
    let lhs = match num_f64(&lhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rhs = match num_f64(&rhs) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::argument::Argument;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, value_from_f64, value_from_i64};
    use tucana::shared::{Value, number_value, value::Kind};

    // ---- helpers: Arguments ----
    fn a_num(n: f64) -> Argument {
        Argument::Eval(value_from_f64(n))
    }
    fn a_int(n: i64) -> Argument {
        Argument::Eval(value_from_i64(n))
    }
    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }

    // ---- helpers: extractors ----
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
    fn expect_int(sig: Signal) -> i64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => match n.number {
                Some(number_value::Number::Integer(i)) => i,
                Some(number_value::Number::Float(f)) => {
                    panic!("Expected Integer NumberValue, got Float({})", f)
                }
                None => panic!("Expected Integer NumberValue, got None"),
            },
            other => panic!("Expected NumberValue, got {:?}", other),
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

    // dummy runner for handlers that accept `run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal`
    fn dummy_run(_: i64, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    #[test]
    fn test_add_and_multiply() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        assert_eq!(
            expect_num(add(&[a_num(5.0), a_num(3.0)], &mut ctx, &mut run)),
            8.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(multiply(&[a_num(4.0), a_num(2.5)], &mut ctx, &mut run)),
            10.0
        );
    }

    #[test]
    fn test_has_digits_and_remove_digits() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert!(!expect_bool(has_digits(&[a_int(42)], &mut ctx, &mut run)));

        let mut run = dummy_run;
        assert!(expect_bool(has_digits(&[a_num(42.5)], &mut ctx, &mut run)));

        let mut run = dummy_run;
        assert_eq!(
            expect_int(remove_digits(&[a_int(123)], &mut ctx, &mut run)),
            123
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_int(remove_digits(&[a_num(12.99)], &mut ctx, &mut run)),
            12
        );
    }

    #[test]
    fn test_substract_and_divide() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(substract(&[a_num(10.0), a_num(4.0)], &mut ctx, &mut run)),
            6.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(divide(&[a_num(15.0), a_num(3.0)], &mut ctx, &mut run)),
            5.0
        );

        // divide by zero -> Failure
        let mut run = dummy_run;
        match divide(&[a_num(10.0), a_num(0.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure on divide by zero, got {:?}", s),
        }
    }

    #[test]
    fn test_modulo_and_abs() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(modulo(&[a_num(10.0), a_num(3.0)], &mut ctx, &mut run)),
            1.0
        );

        // modulo by zero -> Failure
        let mut run = dummy_run;
        match modulo(&[a_num(10.0), a_num(0.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure on modulo by zero, got {:?}", s),
        }

        let mut run = dummy_run;
        assert_eq!(expect_num(abs(&[a_num(-7.5)], &mut ctx, &mut run)), 7.5);
    }

    #[test]
    fn test_comparisons_and_zero() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert!(expect_bool(is_positive(&[a_num(5.0)], &mut ctx, &mut run)));
        let mut run = dummy_run;
        assert!(!expect_bool(is_positive(
            &[a_num(-1.0)],
            &mut ctx,
            &mut run
        )));
        let mut run = dummy_run;
        assert!(expect_bool(is_positive(&[a_num(0.0)], &mut ctx, &mut run)));

        let mut run = dummy_run;
        assert!(expect_bool(is_greater(
            &[a_num(10.0), a_num(5.0)],
            &mut ctx,
            &mut run
        )));
        let mut run = dummy_run;
        assert!(expect_bool(is_less(
            &[a_num(3.0), a_num(7.0)],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        assert!(expect_bool(is_zero(&[a_num(0.0)], &mut ctx, &mut run)));
        let mut run = dummy_run;
        assert!(!expect_bool(is_zero(&[a_num(0.01)], &mut ctx, &mut run)));
    }

    #[test]
    fn test_powers_and_exponential() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(expect_num(square(&[a_num(4.0)], &mut ctx, &mut run)), 16.0);

        let mut run = dummy_run;
        assert_eq!(
            expect_num(exponential(&[a_num(2.0), a_num(3.0)], &mut ctx, &mut run)),
            8.0
        );
    }

    #[test]
    fn test_constants() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert!(
            (expect_num(pi(&[], &mut ctx, &mut run)) - std::f64::consts::PI).abs() < f64::EPSILON
        );

        let mut run = dummy_run;
        assert!(
            (expect_num(euler(&[], &mut ctx, &mut run)) - std::f64::consts::E).abs() < f64::EPSILON
        );

        let mut run = dummy_run;
        let inf = expect_num(infinity(&[], &mut ctx, &mut run));
        assert!(inf.is_infinite() && inf.is_sign_positive());
    }

    #[test]
    fn test_rounding() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(round_up(
                &[a_num(f64::consts::PI), a_num(2.0)],
                &mut ctx,
                &mut run
            )),
            3.15
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(round_down(
                &[a_num(f64::consts::PI), a_num(2.0)],
                &mut ctx,
                &mut run
            )),
            3.14
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(round(&[a_num(3.145), a_num(2.0)], &mut ctx, &mut run)),
            3.15
        );
    }

    #[test]
    fn test_roots_and_logs() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(square_root(&[a_num(16.0)], &mut ctx, &mut run)),
            4.0
        );

        // cube root via exponent 1/3
        let mut run = dummy_run;
        let r = expect_num(root(&[a_num(8.0), a_num(1.0 / 3.0)], &mut ctx, &mut run));
        assert!((r - 2.0).abs() < 1e-6);

        let mut run = dummy_run;
        let lg = expect_num(log(&[a_num(100.0), a_num(10.0)], &mut ctx, &mut run));
        assert!((lg - 2.0).abs() < f64::EPSILON);

        let mut run = dummy_run;
        let ln1 = expect_num(ln(&[a_num(f64::consts::E)], &mut ctx, &mut run));
        assert!((ln1 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_text_conversions() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(from_text(&[a_str("42.5")], &mut ctx, &mut run)),
            42.5
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(as_text(&[a_num(42.5)], &mut ctx, &mut run)),
            "42.5".to_string()
        );

        // from_text failure
        let mut run = dummy_run;
        match from_text(&[a_str("not_a_number")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for invalid parse, got {:?}", s),
        }
    }

    #[test]
    fn test_min_max_and_negate() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(min(&[a_num(3.0), a_num(7.0)], &mut ctx, &mut run)),
            3.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(max(&[a_num(3.0), a_num(7.0)], &mut ctx, &mut run)),
            7.0
        );

        let mut run = dummy_run;
        assert_eq!(expect_num(negate(&[a_num(5.0)], &mut ctx, &mut run)), -5.0);
    }

    #[test]
    fn test_random_range() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let r = expect_num(random(&[a_num(1.0), a_num(10.0)], &mut ctx, &mut run));
        assert!(r >= 1.0 && r < 10.0);
    }

    #[test]
    fn test_random_range_numbers_equal() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let r = expect_num(random(&[a_num(1.0), a_num(1.0)], &mut ctx, &mut run));
        assert!(r == 1.0);
    }

    #[test]
    fn test_random_range_fist_bigger_then_second() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let res = random(&[a_num(10.0), a_num(1.0)], &mut ctx, &mut run);
        assert!(matches!(res, Signal::Failure(_)));
    }

    #[test]
    fn test_trig_and_hyperbolic() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let s = expect_num(sin(&[a_num(f64::consts::PI / 2.0)], &mut ctx, &mut run));
        assert!((s - 1.0).abs() < 1e-12);

        let mut run = dummy_run;
        let c = expect_num(cos(&[a_num(0.0)], &mut ctx, &mut run));
        assert!((c - 1.0).abs() < 1e-12);

        let mut run = dummy_run;
        let t = expect_num(tan(&[a_num(f64::consts::PI / 4.0)], &mut ctx, &mut run));
        assert!((t - 1.0).abs() < 1e-4);

        let mut run = dummy_run;
        let asn = expect_num(arcsin(&[a_num(1.0)], &mut ctx, &mut run));
        assert!((asn - f64::consts::PI / 2.0).abs() < 1e-12);

        let mut run = dummy_run;
        let acs = expect_num(arccos(&[a_num(1.0)], &mut ctx, &mut run));
        assert!(acs.abs() < 1e-12);

        let mut run = dummy_run;
        let atn = expect_num(arctan(&[a_num(1.0)], &mut ctx, &mut run));
        assert!((atn - f64::consts::PI / 4.0).abs() < 1e-12);

        let mut run = dummy_run;
        let sh = expect_num(sinh(&[a_num(0.0)], &mut ctx, &mut run));
        assert!(sh.abs() < 1e-12);

        let mut run = dummy_run;
        let ch = expect_num(cosh(&[a_num(0.0)], &mut ctx, &mut run));
        assert!((ch - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_clamp_and_is_equal() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(clamp(
                &[a_num(5.0), a_num(1.0), a_num(10.0)],
                &mut ctx,
                &mut run
            )),
            5.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(clamp(
                &[a_num(-5.0), a_num(1.0), a_num(10.0)],
                &mut ctx,
                &mut run
            )),
            1.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(clamp(
                &[a_num(15.0), a_num(1.0), a_num(10.0)],
                &mut ctx,
                &mut run
            )),
            10.0
        );

        let mut run = dummy_run;
        assert!(expect_bool(is_equal(
            &[a_num(5.0), a_num(5.0)],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        assert!(!expect_bool(is_equal(
            &[a_num(5.0), a_num(3.0)],
            &mut ctx,
            &mut run
        )));
    }
}
