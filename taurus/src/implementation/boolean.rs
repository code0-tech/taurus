use crate::context::argument::Argument;
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::{context::Context, error::RuntimeError};
use tucana::shared::{Value, value::Kind};

pub fn collect_boolean_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("std::boolean::as_number", HandlerFn::eager(as_number, 1)),
        ("std::boolean::as_text", HandlerFn::eager(as_text, 1)),
        (
            "std::boolean::from_number",
            HandlerFn::eager(from_number, 1),
        ),
        ("std::boolean::from_text", HandlerFn::eager(from_text, 1)),
        ("std::boolean::is_equal", HandlerFn::eager(is_equal, 2)),
        ("std::boolean::negate", HandlerFn::eager(negate, 1)),
    ]
}

fn as_number(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: bool);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue((value as i64) as f64)),
    })
}

fn as_text(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: bool);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn from_number(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => number: f64);
    let is_zero = number == 0.0;
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!is_zero)),
    })
}

fn from_text(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => text: String);

    match text.to_lowercase().parse::<bool>() {
        Ok(b) => Signal::Success(Value {
            kind: Some(Kind::BoolValue(b)),
        }),
        Err(_) => Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Failed to parse boolean from string: {:?}", text),
        )),
    }
}

fn is_equal(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: bool, rhs: bool);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

fn negate(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: bool);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!value)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use tucana::shared::{Value, value::Kind};

    // ---- helpers: make Arguments ----
    fn a_bool(b: bool) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(b)),
        })
    }
    fn a_num(n: f64) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::NumberValue(n)),
        })
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
            }) => n,
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
    fn dummy_run(_: i64) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::BoolValue(true)),
        })
    }

    // ---- tests ----

    #[test]
    fn test_as_number_success() {
        let mut ctx = Context::new();
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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
        let mut ctx = Context::new();

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
