//! Boolean standard-library handlers.
//!
//! This module keeps conversions explicit (`from_text`, `as_number`, ...) so flow behavior
//! stays predictable across local and remote execution targets.

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::value_from_i64;
use tucana::shared::{Value, value::Kind};

taurus_macros::module! {
    identifier = "taurus-boolean",
    name(en_US = "Boolean"),
    description(en_US = "Work with Booleans."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:toggle-left",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "BOOLEAN",
    module = "taurus-boolean",
    name(en_US = "Boolean"),
    display_message(en_US = "Boolean"),
    alias(en_US = "bool;boolean;bit"),
    type_string = "boolean",
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::as_number",
    module = "taurus-boolean",
    signature = "(value: BOOLEAN): NUMBER",
    name(en_US = "Boolean as Number"),
    description(en_US = "Will convert the boolean to a number."),
    display_message(en_US = "Convert ${value} to number"),
    alias(en_US = "to number;numeric;boolean;logic;std;as;number"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["BOOLEAN", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "Converts a boolean to a number.")
)]
fn as_number(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: bool);
    Signal::Success(value_from_i64(if value { 1 } else { 0 }))
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::as_text",
    module = "taurus-boolean",
    signature = "(value: BOOLEAN): TEXT",
    name(en_US = "Boolean as Text"),
    description(en_US = "Will convert the boolean to text."),
    display_message(en_US = "Convert ${value} to text"),
    alias(en_US = "to text;string;format number;boolean;logic;std;as;text"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["BOOLEAN", "TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "Converts a boolean to a text.")
)]
fn as_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: bool);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::from_number",
    module = "taurus-boolean",
    signature = "(value: NUMBER): BOOLEAN",
    name(en_US = "Boolean from Number"),
    description(en_US = "Will convert the number to a boolean."),
    display_message(en_US = "Convert ${value} to boolean"),
    alias(en_US = "from number;to boolean;convert;boolean;logic;std;from;number"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["BOOLEAN", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "Converts a number to a boolean.")
)]
fn from_number(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => number: f64);
    let is_zero = number == 0.0;
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!is_zero)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::from_text",
    module = "taurus-boolean",
    signature = "(value: TEXT): BOOLEAN",
    name(en_US = "Boolean from Text"),
    description(en_US = "Will convert the string to a boolean."),
    display_message(en_US = "Convert ${value} to boolean"),
    alias(en_US = "from text;parse;convert;boolean;logic;std;from;text"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["TEXT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "Converts a text to a boolean.")
)]
fn from_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => text: String);

    // Keep parsing strict to Rust's bool format (`true` / `false`) after lowercase normalization.
    match text.to_lowercase().parse::<bool>() {
        Ok(b) => Signal::Success(Value {
            kind: Some(Kind::BoolValue(b)),
        }),
        Err(_) => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Failed to parse boolean from string: {:?}", text),
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::is_equal",
    module = "taurus-boolean",
    signature = "(first: BOOLEAN, second: BOOLEAN): BOOLEAN",
    name(en_US = "Is Equal"),
    description(
        en_US = "Compares two boolean values for equality. Returns true if they are the same, false otherwise."
    ),
    display_message(en_US = "${first} Equals ${second}"),
    alias(en_US = "equal;equals;same;boolean;logic;std;is"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["BOOLEAN"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First"),
    description(en_US = "The first boolean value to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second"),
    description(en_US = "The second boolean value to compare.")
)]
fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => lhs: bool, rhs: bool);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::boolean::negate",
    module = "taurus-boolean",
    signature = "(value: BOOLEAN): BOOLEAN",
    name(en_US = "Negate Boolean"),
    description(en_US = "Negates a boolean value."),
    display_message(en_US = "Negate ${value}"),
    alias(en_US = "negate;negative;invert;opposite;boolean;logic;std"),
    display_icon = "tabler:toggle-left",
    linked_data_type_identifiers = ["BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The boolean value to negate.")
)]
fn negate(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: bool);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!value)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, value_from_f64};
    use tucana::shared::{Value, value::Kind};

    // ---- helpers: make Arguments ----
    fn a_bool(b: bool) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(b)),
        })
    }
    fn a_num(n: f64) -> Argument {
        Argument::Eval(value_from_f64(n))
    }
    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }

    // ---- helpers: unwrap Signal ----
    fn expect_num(sig: Signal) -> f64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_f64(&n).unwrap_or_default(),
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
    fn expect_bool(sig: Signal) -> bool {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::BoolValue(b)),
            }) => b,
            other => panic!("Expected BoolValue, got {:?}", other),
        }
    }

    // dummy `run` closure (unused by these handlers)
    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::BoolValue(true)),
        })
    }

    // ---- tests ----

    #[test]
    fn test_as_number_success() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        assert_eq!(
            expect_num(as_number(&[a_bool(true)], &mut ctx, &mut run)),
            1.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(as_number(&[a_bool(false)], &mut ctx, &mut run)),
            0.0
        );
    }

    #[test]
    fn test_as_number_errors() {
        let mut ctx = ValueStore::default();

        // wrong arity: none
        let mut run = dummy_run;
        match as_number(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        // wrong type
        let mut run = dummy_run;
        match as_number(&[a_num(1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-bool, got {:?}", s),
        }

        // too many args
        let mut run = dummy_run;
        match as_number(&[a_bool(true), a_bool(false)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 2, got {:?}", s),
        }
    }

    #[test]
    fn test_as_text_success() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(as_text(&[a_bool(true)], &mut ctx, &mut run)),
            "true"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(as_text(&[a_bool(false)], &mut ctx, &mut run)),
            "false"
        );
    }

    #[test]
    fn test_as_text_errors() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        match as_text(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        let mut run = dummy_run;
        match as_text(&[a_num(5.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-bool, got {:?}", s),
        }

        let mut run = dummy_run;
        match as_text(&[a_bool(true), a_bool(false)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 2, got {:?}", s),
        }
    }

    #[test]
    fn test_from_number_success() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_number(&[a_num(0.0)], &mut ctx, &mut run)),
            false
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_number(&[a_num(3.5)], &mut ctx, &mut run)),
            true
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_number(&[a_num(-2.0)], &mut ctx, &mut run)),
            true
        );

        // -0.0 should behave like 0.0
        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_number(&[a_num(-0.0)], &mut ctx, &mut run)),
            false
        );
    }

    #[test]
    fn test_from_number_errors() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        match from_number(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        let mut run = dummy_run;
        match from_number(&[a_bool(true)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-number, got {:?}", s),
        }

        let mut run = dummy_run;
        match from_number(&[a_num(1.0), a_num(2.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 2, got {:?}", s),
        }
    }

    #[test]
    fn test_from_text_success_and_errors() {
        let mut ctx = ValueStore::default();

        // success (case-insensitive)
        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_text(&[a_str("true")], &mut ctx, &mut run)),
            true
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(from_text(&[a_str("FALSE")], &mut ctx, &mut run)),
            false
        );

        // errors
        let mut run = dummy_run;
        match from_text(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        let mut run = dummy_run;
        match from_text(&[a_str("yes")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for unparseable bool, got {:?}", s),
        }

        let mut run = dummy_run;
        match from_text(&[a_num(1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-string, got {:?}", s),
        }

        let mut run = dummy_run;
        match from_text(&[a_str("true"), a_str("false")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 2, got {:?}", s),
        }
    }

    #[test]
    fn test_is_equal_and_errors() {
        let mut ctx = ValueStore::default();

        // equalities
        let mut run = dummy_run;
        assert_eq!(
            expect_bool(is_equal(&[a_bool(true), a_bool(true)], &mut ctx, &mut run)),
            true
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(is_equal(
                &[a_bool(false), a_bool(false)],
                &mut ctx,
                &mut run
            )),
            true
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(is_equal(&[a_bool(true), a_bool(false)], &mut ctx, &mut run)),
            false
        );

        // arity/type errors
        let mut run = dummy_run;
        match is_equal(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        let mut run = dummy_run;
        match is_equal(&[a_bool(true)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 1, got {:?}", s),
        }

        let mut run = dummy_run;
        match is_equal(&[a_bool(true), a_num(1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-bool rhs, got {:?}", s),
        }
    }

    #[test]
    fn test_negate_success_and_errors() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(negate(&[a_bool(true)], &mut ctx, &mut run)),
            false
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_bool(negate(&[a_bool(false)], &mut ctx, &mut run)),
            true
        );

        // errors
        let mut run = dummy_run;
        match negate(&[], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 0, got {:?}", s),
        }

        let mut run = dummy_run;
        match negate(&[a_num(1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for non-bool, got {:?}", s),
        }

        let mut run = dummy_run;
        match negate(&[a_bool(true), a_bool(false)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for arity 2, got {:?}", s),
        }
    }
}
