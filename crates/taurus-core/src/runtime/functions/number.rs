//! Numeric standard-library handlers.
//!
//! Most operators keep an integer fast-path (checked ops) and fall back to `f64` arithmetic
//! when needed so common integer-heavy flows avoid unnecessary float conversion.

use std::f64;

use tucana::shared::helper::value::ToValue;
use tucana::shared::{NumberValue, Value, number_value, value::Kind};

use crate::handler::argument::Argument;
use crate::handler::macros::{args, no_args};
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

taurus_macros::module! {
    identifier = "taurus-number",
    name(en_US = "Number"),
    description(en_US = "Work with Numbers."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:math-function",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "NUMBER",
    module = "taurus-number",
    name(en_US = "Number"),
    display_message(en_US = "Number"),
    alias(en_US = "number;integer;float;double;long"),
    type_string = "number",
}

#[taurus_macros::runtime_function(
    identifier = "std::number::has_digits",
    module = "taurus-number",
    signature = "(number: NUMBER): BOOLEAN",
    name(en_US = "Has Digits in Number"),
    description(en_US = "Checks if the given number contains any digit characters"),
    display_message(en_US = "Does ${number} have digits"),
    alias(en_US = "has;digits;contains;number;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "number",
    name(en_US = "Number Input"),
    description(en_US = "The number to check for digit characters.")
)]
fn has_digits(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::remove_digits",
    module = "taurus-number",
    signature = "(number: NUMBER): NUMBER",
    name(en_US = "Remove Digits from Number"),
    description(en_US = "Removes all digit characters from the input number, effectively stripping it down to its non-digit components."),
    display_message(en_US = "Remove Digits from ${number}"),
    alias(en_US = "remove:digits;strip;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "number",
    name(en_US = "Number Input"),
    description(
        en_US = "This is the numeric input. The result will be its value without any digits."
    )
)]
fn remove_digits(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::add",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Add Numbers"),
    description(en_US = "Adds two numbers together."),
    display_message(en_US = "${first} Plus ${second}"),
    alias(en_US = "add;plus;sum;total;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(en_US = "The first number to add.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "The second number to add.")
)]
fn add(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::multiply",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Multiply"),
    description(en_US = "Takes two numeric inputs and returns their product."),
    display_message(en_US = "${first} Multiply by ${second}"),
    alias(en_US = "multiply;times;product;mul;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(en_US = "The first number to multiply.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "The second number to multiply.")
)]
fn multiply(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::subtract",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Subtract"),
    description(en_US = "Subtracts the second number from the first number."),
    display_message(en_US = "${first} Minus ${second}"),
    alias(en_US = "subtract;minus;difference;sub;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "Minuend"),
    description(
        en_US = "The number from which another number (the subtrahend) is to be subtracted."
    )
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Subtrahend"),
    description(en_US = "The number to subtract from the first number (the minuend).")
)]
fn subtract(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::divide",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Divide Numbers"),
    description(
        en_US = "Returns the result of dividing the first numeric input (dividend) by the second (divisor)."
    ),
    display_message(en_US = "${first} Divided by ${second}"),
    alias(en_US = "divide;division;quotient;div;number;math;std"),
    display_icon = "tabler:math-function",
    throws_error
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "Dividend"),
    description(
        en_US = "This is the numerator or the number that will be divided by the second value."
    )
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Divisor"),
    description(en_US = "This is the denominator or the value that divides the first number.")
)]
fn divide(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::modulo",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Modulo"),
    description(en_US = "Computes the modulus (remainder) of dividing the first numeric input by the second."),
    display_message(en_US = "${first} Modulus ${second}"),
    alias(en_US = "modulo;mod;remainder;modulus;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "Number"),
    description(en_US = "The number to apply the modulo operator onto.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Modulo"),
    description(en_US = "The modulo operator.")
)]
fn modulo(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::abs",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Absolute Value"),
    description(en_US = "Removes the sign from the input number, returning its non-negative value."),
    display_message(en_US = "Absolute Value of ${value}"),
    alias(en_US = "absolute;abs;magnitude;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(
        en_US = "This is the numeric input. The result will be its absolute (non-negative) value."
    )
)]
fn abs(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::is_positive",
    module = "taurus-number",
    signature = "(value: NUMBER): BOOLEAN",
    name(en_US = "Is Positive Number"),
    description(en_US = "Evaluates the input number and returns true if it is positive (greater than zero), otherwise false."),
    display_message(en_US = "${value} Is Greater than 0"),
    alias(en_US = "positive;greater than zero;number;math;std;is"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The number to check for positivity.")
)]
fn is_positive(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::is_greater",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): BOOLEAN",
    name(en_US = "Is Greater"),
    description(en_US = "Returns true if the first numeric input is greater than the second; otherwise, returns false."),
    display_message(en_US = "${first} Is Greater than ${second}"),
    alias(en_US = "greater;larger;more;number;math;std;is"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["BOOLEAN", "NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(
        en_US = "This is the number that will be evaluated to determine if it is greater than the second number."
    )
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "This is the number that the first number will be compared to.")
)]
fn is_greater(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::is_less",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): BOOLEAN",
    name(en_US = "Is Less"),
    description(en_US = "Returns true if the first numeric input is less than the second; otherwise, returns false."),
    display_message(en_US = "${first} Less than ${second}"),
    alias(en_US = "less;smaller;fewer;number;math;std;is"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(
        en_US = "This is the number that will be evaluated to determine if it is less than the second number."
    )
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "This is the number that the first number will be compared to.")
)]
fn is_less(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::is_zero",
    module = "taurus-number",
    signature = "(value: NUMBER): BOOLEAN",
    name(en_US = "Number Is Zero"),
    description(en_US = "Returns true if the input number is zero. Otherwise returns false."),
    display_message(en_US = "${value} Equals 0"),
    alias(en_US = "zero;equals zero;number;math;std;is"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(
        en_US = "This is the numeric input evaluated to determine whether it equals zero."
    )
)]
fn is_zero(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::square",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Square"),
    description(en_US = "Returns the square of the given number."),
    display_message(en_US = "${value} Squared"),
    alias(en_US = "square;squared;power two;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The number to be squared.")
)]
fn square(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::exponential",
    module = "taurus-number",
    signature = "(base: NUMBER, exponent: NUMBER): NUMBER",
    name(en_US = "Exponential"),
    description(en_US = "Computes the result of raising the base to the power specified by the exponent."),
    display_message(en_US = "${base} to the Exponent of ${exponent}"),
    alias(en_US = "exponential;exp;e power;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "base",
    name(en_US = "Base"),
    description(
        en_US = "This is the numeric value that will be raised to the power of the exponent."
    )
)]
#[parameter(
    runtime_name = "exponent",
    name(en_US = "Exponent"),
    description(en_US = "This numeric value indicates the power to which the base is raised.")
)]
fn exponential(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::pi",
    module = "taurus-number",
    signature = "(): NUMBER",
    name(en_US = "Pi"),
    description(en_US = "Provides the constant value of pi, approximately 3.14159, used in many mathematical calculations."),
    display_message(en_US = "Pi"),
    alias(en_US = "pi;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
fn pi(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_f64(f64::consts::PI))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::euler",
    module = "taurus-number",
    signature = "(): NUMBER",
    name(en_US = "Euler's Number"),
    description(en_US = "Provides the constant value of Euler's number, approximately 2.71828, which is the base of the natural logarithm."),
    display_message(en_US = "Euler's Number"),
    alias(en_US = "euler;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
fn euler(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_f64(f64::consts::E))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::round_up",
    module = "taurus-number",
    signature = "(value: NUMBER, decimals: NUMBER): NUMBER",
    name(en_US = "Round Up"),
    description(en_US = "Performs rounding on the given value, always rounding up to the nearest value at the given decimal precision."),
    display_message(en_US = "Round Upwards ${value} with ${decimals} Decimal Places"),
    alias(en_US = "round up;ceil;ceiling;number;math;std;round;up"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(en_US = "The number to be rounded up.")
)]
#[parameter(
    runtime_name = "decimals",
    name(en_US = "Decimal Places"),
    description(en_US = "The number of decimal places to round up to.")
)]
fn round_up(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::round_down",
    module = "taurus-number",
    signature = "(value: NUMBER, decimals: NUMBER): NUMBER",
    name(en_US = "Round Number Down"),
    description(en_US = "Rounds a number downward to the specified number of decimal places."),
    display_message(en_US = "Round Down ${value} with ${decimals} Decimal Places"),
    alias(en_US = "round down;floor;number;math;std;round;down"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The numeric input that will be rounded downwards.")
)]
#[parameter(
    runtime_name = "decimals",
    name(en_US = "Decimal Places"),
    description(en_US = "Specifies how many decimal digits to keep after rounding down.")
)]
fn round_down(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::round",
    module = "taurus-number",
    signature = "(value: NUMBER, decimals: NUMBER): NUMBER",
    name(en_US = "Round Number"),
    description(en_US = "Rounds a number to the nearest value at the specified number of decimal places."),
    display_message(en_US = "Round ${value} with ${decimals} Decimal Places"),
    alias(en_US = "round;nearest;approximate;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The numeric input that will be rounded to the nearest value.")
)]
#[parameter(
    runtime_name = "decimals",
    name(en_US = "Decimal Places"),
    description(en_US = "Specifies how many decimal digits to keep after rounding.")
)]
fn round(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::square_root",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Square Root"),
    description(en_US = "Calculates the positive square root of the input number."),
    display_message(en_US = "Square Root of ${value}"),
    alias(en_US = "square root;sqrt;root;number;math;std;square"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The number to find the square root of.")
)]
fn square_root(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sqrt()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::root",
    module = "taurus-number",
    signature = "(value: NUMBER, root_exponent: NUMBER): NUMBER",
    name(en_US = "Root"),
    description(en_US = "Calculates the nth root of the input number, where n is specified by the root exponent."),
    display_message(en_US = "${root_exponent} Root of ${value}"),
    alias(en_US = "root;nth root;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Value"),
    description(en_US = "The numeric input for which the root will be calculated.")
)]
#[parameter(
    runtime_name = "root_exponent",
    name(en_US = "Root Exponent"),
    description(en_US = "The degree of the root to extract.")
)]
fn root(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::log",
    module = "taurus-number",
    signature = "(value: NUMBER, base: NUMBER): NUMBER",
    name(en_US = "Logarithm"),
    description(en_US = "Calculates and returns the logarithm of a number with respect to a specified base."),
    display_message(en_US = "Logarithm with Base ${base} of ${value}"),
    alias(en_US = "log;logarithm;log base;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The numeric input whose logarithm is to be calculated.")
)]
#[parameter(
    runtime_name = "base",
    name(en_US = "Base"),
    description(
        en_US = "Specifies the logarithmic base to use (e.g., 10 for common log, e for natural log)."
    )
)]
fn log(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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
    if !value.is_finite() || !base.is_finite() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Log input and base must be finite numbers",
        ));
    }
    if value <= 0.0 {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Log input must be greater than zero",
        ));
    }
    if base <= 0.0 || base == 1.0 {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Log base must be greater than zero and not equal to one",
        ));
    }
    let result = value.log(base);
    if !result.is_finite() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Log result was not finite",
        ));
    }
    Signal::Success(value_from_f64(result))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::ln",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Natural Logarithm"),
    description(en_US = "Calculates the natural logarithm (log base e) of a number."),
    display_message(en_US = "Natural Logarithm of ${value}"),
    alias(en_US = "natural log;ln;log e;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Number"),
    description(
        en_US = "The numeric input whose natural logarithm (log base e) will be calculated."
    )
)]
fn ln(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.ln()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::from_text",
    module = "taurus-number",
    signature = "(text: TEXT): NUMBER",
    name(en_US = "Number from Text"),
    description(en_US = "Attempts to parse the provided text input and return its numeric equivalent."),
    display_message(en_US = "Convert ${text} to Number"),
    alias(en_US = "from text;parse;convert;number;math;std;from;text"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "TEXT"],
)]
#[parameter(
    runtime_name = "text",
    name(en_US = "Text"),
    description(en_US = "The text string to convert to a number.")
)]
fn from_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::as_text",
    module = "taurus-number",
    signature = "(number: NUMBER): TEXT",
    name(en_US = "Number as Text"),
    description(en_US = "Converts a number into text."),
    display_message(en_US = "Convert ${number} to Text"),
    alias(en_US = "to text;string;format number;number;math;std;as;text"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "TEXT"],
)]
#[parameter(
    runtime_name = "number",
    name(en_US = "Number"),
    description(en_US = "The number to convert to text.")
)]
fn as_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::min",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Minimum"),
    description(en_US = "Compares two numbers and returns the minimum value."),
    display_message(en_US = "Minimum of ${first} and ${second}"),
    alias(en_US = "min;minimum;smallest;least;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(en_US = "The first number to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "The second number to compare.")
)]
fn min(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::max",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): NUMBER",
    name(en_US = "Maximum Number"),
    description(en_US = "Compares two numbers and returns the maximum value."),
    display_message(en_US = "Maximum of ${first} and ${second}"),
    alias(en_US = "max;maximum;largest;greatest;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(en_US = "The first number to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "The second number to compare.")
)]
fn max(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::negate",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Negate"),
    description(en_US = "Returns the negation of a number (multiplies by -1)."),
    display_message(en_US = "Negate ${value}"),
    alias(en_US = "negate;negative;invert;opposite;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The number to negate.")
)]
fn negate(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::random_number",
    module = "taurus-number",
    signature = "(min: NUMBER, max: NUMBER): NUMBER",
    name(en_US = "Random Number"),
    description(en_US = "Returns a randomly generated number within the given range, inclusive of both minimum and maximum."),
    display_message(en_US = "Random Number Between ${min} and ${max}"),
    alias(en_US = "random;rand;random number;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "min",
    name(en_US = "Minimum Value"),
    description(en_US = "Defines the lower bound (inclusive) for the random number generation.")
)]
#[parameter(
    runtime_name = "max",
    name(en_US = "Maximum Value"),
    description(en_US = "Defines the upper bound (inclusive) for the random number generation.")
)]
fn random(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::sin",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Sine"),
    description(en_US = "Calculates the sine of the input value."),
    display_message(en_US = "Sine of ${value}"),
    alias(en_US = "sin;sine;trigonometry;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(en_US = "The number for which to calculate the sine.")
)]
fn sin(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sin()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::cos",
    module = "taurus-number",
    signature = "(radians: NUMBER): NUMBER",
    name(en_US = "Cosine"),
    description(en_US = "Calculates the cosine value of the input angle measured in radians."),
    display_message(en_US = "Cosine of ${radians}"),
    alias(en_US = "cos;cosine;trigonometry;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "radians",
    name(en_US = "Radians"),
    description(en_US = "Computes the cosine of the given angle in radians.")
)]
fn cos(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.cos()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::tan",
    module = "taurus-number",
    signature = "(radians: NUMBER): NUMBER",
    name(en_US = "Tangent"),
    description(en_US = "Calculates the tangent value of the input angle measured in radians."),
    display_message(en_US = "Tangent of ${radians}"),
    alias(en_US = "tan;tangent;trigonometry;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "radians",
    name(en_US = "Radians"),
    description(en_US = "Computes the tangent of the given angle in radians.")
)]
fn tan(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.tan()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::arcsin",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Arcsine"),
    description(en_US = "Computes the angle in radians whose sine is the given number."),
    display_message(en_US = "Arcsine of ${value}"),
    alias(en_US = "arcsin;asin;inverse sine;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "Calculates the arcsine (inverse sine) of the input value.")
)]
fn arcsin(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.asin()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::arccos",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Arccosine"),
    description(en_US = "Computes the angle in radians whose cosine is the given number."),
    display_message(en_US = "Arccosine of ${value}"),
    alias(en_US = "arccos;acos;inverse cosine;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Number"),
    description(en_US = "Calculates the arccosine (inverse cosine) of the input value.")
)]
fn arccos(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.acos()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::arctan",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Arctangent"),
    description(en_US = "Computes the angle in radians whose tangent is the given number."),
    display_message(en_US = "Arctangent of ${value}"),
    alias(en_US = "arctan;atan;inverse tangent;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Value"),
    description(en_US = "Calculates the arctangent (inverse tangent) of the input value.")
)]
fn arctan(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.atan()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::sinh",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Hyperbolic Sine"),
    description(en_US = "Calculates the hyperbolic sine (sinh) of the input value."),
    display_message(en_US = "Hyperbolic Sine of ${value}"),
    alias(en_US = "sinh;hyperbolic sine;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(en_US = "The number for which to calculate the hyperbolic sine.")
)]
fn sinh(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.sinh()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::cosh",
    module = "taurus-number",
    signature = "(value: NUMBER): NUMBER",
    name(en_US = "Hyperbolic Cosine"),
    description(en_US = "Calculates the hyperbolic cosine (cosh) of the input value."),
    display_message(en_US = "Hyperbolic Cosine of ${value}"),
    alias(en_US = "cosh;hyperbolic cosine;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(en_US = "The number for which to calculate the hyperbolic cosine.")
)]
fn cosh(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: NumberValue);
    let value = match num_f64(&value) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Signal::Success(value_from_f64(value.cosh()))
}

#[taurus_macros::runtime_function(
    identifier = "std::number::clamp",
    module = "taurus-number",
    signature = "(value: NUMBER, min: NUMBER, max: NUMBER): NUMBER",
    name(en_US = "Clamp Number"),
    description(en_US = "Returns the given number clamped between the minimum and maximum bounds."),
    display_message(en_US = "Clamp ${value} between ${min} and ${max}"),
    alias(en_US = "clamp;limit;bound;number;math;std"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Number Input"),
    description(en_US = "The input number that will be limited (clamped) to the specified range.")
)]
#[parameter(
    runtime_name = "min",
    name(en_US = "Minimum"),
    description(en_US = "The minimum allowed value in the clamping operation.")
)]
#[parameter(
    runtime_name = "max",
    name(en_US = "Maximum"),
    description(en_US = "The maximum allowed value in the clamping operation.")
)]
fn clamp(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

#[taurus_macros::runtime_function(
    identifier = "std::number::is_equal",
    module = "taurus-number",
    signature = "(first: NUMBER, second: NUMBER): BOOLEAN",
    name(en_US = "Is Equal"),
    description(en_US = "Returns true if the first number is equal to the second number, otherwise false."),
    display_message(en_US = "${first} Equals ${second}"),
    alias(en_US = "equal;equals;same;number;math;std;is"),
    display_icon = "tabler:math-function",
    linked_data_type_identifiers = ["NUMBER", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Number"),
    description(en_US = "The first number to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Number"),
    description(en_US = "The second number to compare.")
)]
fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
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

    // dummy runner for handlers that accept `run: &mut crate::handler::registry::ThunkRunner<'_>`
    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
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
    fn test_subtract_and_divide() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(subtract(&[a_num(10.0), a_num(4.0)], &mut ctx, &mut run)),
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

        let mut run = dummy_run;
        assert!(matches!(
            log(&[a_num(-100.0), a_num(10.0)], &mut ctx, &mut run),
            Signal::Failure(_)
        ));

        let mut run = dummy_run;
        assert!(matches!(
            log(&[a_num(100.0), a_num(1.0)], &mut ctx, &mut run),
            Signal::Failure(_)
        ));

        let mut run = dummy_run;
        assert!(matches!(
            log(&[a_num(100.0), a_num(0.0)], &mut ctx, &mut run),
            Signal::Failure(_)
        ));
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
