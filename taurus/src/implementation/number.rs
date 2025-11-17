use std::f64;

use tucana::shared::{Value, value::Kind};

use crate::context::argument::Argument;
use crate::context::macros::{args, no_args};
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::{context::context::Context, error::RuntimeError};

pub fn collect_number_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("std::number::add", HandlerFn::eager(add, 2)),
        ("std::number::multiply", HandlerFn::eager(multiply, 2)),
        ("std::number::substract", HandlerFn::eager(substract, 2)),
        ("std::number::divide", HandlerFn::eager(divide, 2)),
        ("std::number::modulo", HandlerFn::eager(modulo, 2)),
        ("std::number::abs", HandlerFn::eager(abs, 1)),
        ("std::number::is_positive", HandlerFn::eager(is_positive, 1)),
        ("std::number::is_greater", HandlerFn::eager(is_greater, 2)),
        ("std::number::is_less", HandlerFn::eager(is_less, 2)),
        ("std::number::is_zero", HandlerFn::eager(is_zero, 1)),
        ("std::number::square", HandlerFn::eager(square, 2)),
        ("std::number::exponential", HandlerFn::eager(exponential, 2)),
        ("std::number::pi", HandlerFn::eager(pi, 0)),
        ("std::number::euler", HandlerFn::eager(euler, 0)),
        ("std::number::infinity", HandlerFn::eager(infinity, 0)),
        ("std::number::round_up", HandlerFn::eager(round_up, 2)),
        ("std::number::round_down", HandlerFn::eager(round_down, 2)),
        ("std::number::round", HandlerFn::eager(round, 2)),
        ("std::number::square_root", HandlerFn::eager(square_root, 1)),
        ("std::number::root", HandlerFn::eager(root, 2)),
        ("std::number::log", HandlerFn::eager(log, 2)),
        ("std::number::ln", HandlerFn::eager(ln, 1)),
        ("std::number::from_text", HandlerFn::eager(from_text, 1)),
        ("std::number::as_text", HandlerFn::eager(as_text, 1)),
        ("std::number::min", HandlerFn::eager(min, 2)),
        ("std::number::max", HandlerFn::eager(max, 2)),
        ("std::number::negate", HandlerFn::eager(negate, 1)),
        ("std::number::random", HandlerFn::eager(random, 2)),
        ("std::number::sin", HandlerFn::eager(sin, 1)),
        ("std::number::cos", HandlerFn::eager(cos, 1)),
        ("std::number::tan", HandlerFn::eager(tan, 1)),
        ("std::number::arcsin", HandlerFn::eager(arcsin, 1)),
        ("std::number::arccos", HandlerFn::eager(arccos, 1)),
        ("std::number::arctan", HandlerFn::eager(arctan, 1)),
        ("std::number::sinh", HandlerFn::eager(sinh, 1)),
        ("std::number::cosh", HandlerFn::eager(cosh, 1)),
        ("std::number::clamp", HandlerFn::eager(clamp, 3)),
        ("std::number::is_equal", HandlerFn::eager(is_equal, 2)),
    ]
}

fn add(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs + rhs)),
    })
}

fn multiply(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs * rhs)),
    })
}

fn substract(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs - rhs)),
    })
}

fn divide(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);

    if rhs == 0.0 {
        return Signal::Failure(RuntimeError::simple_str(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs / rhs)),
    })
}

fn modulo(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);

    if rhs == 0.0 {
        return Signal::Failure(RuntimeError::simple_str(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs % rhs)),
    })
}

fn abs(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.abs())),
    })
}

fn is_positive(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(!value.is_sign_negative())),
    })
}

fn is_greater(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs > rhs)),
    })
}

fn is_less(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs < rhs)),
    })
}

fn is_zero(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value == 0.0)),
    })
}

fn square(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.powf(2.0))),
    })
}

fn exponential(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => base: f64, exponent: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(base.powf(exponent))),
    })
}

fn pi(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    no_args!(args);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(f64::consts::PI)),
    })
}

fn euler(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    no_args!(args);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(f64::consts::E)),
    })
}

fn infinity(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    no_args!(args);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(f64::INFINITY)),
    })
}

fn round_up(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64, decimal_places: f64);
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue((value * factor).ceil() / factor)),
    })
}

fn round_down(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => value: f64, decimal_places: f64);
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue((value * factor).floor() / factor)),
    })
}

fn round(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64, decimal_places: f64);
    let factor = 10_f64.powi(decimal_places as i32);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue((value * factor).round() / factor)),
    })
}

fn square_root(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64) -> Signal,
) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.sqrt())),
    })
}

fn root(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64, root: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.powf(root))),
    })
}

fn log(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64, base: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.log(base))),
    })
}

fn ln(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.ln())),
    })
}

fn from_text(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => string_value: String);

    match string_value.parse::<f64>() {
        Ok(v) => Signal::Success(Value {
            kind: Some(Kind::NumberValue(v)),
        }),
        Err(_) => Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Failed to parse string as number: {}", string_value),
        )),
    }
}

fn as_text(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn min(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs.min(rhs))),
    })
}

fn max(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(lhs.max(rhs))),
    })
}

fn negate(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(-value)),
    })
}

fn random(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => min: f64, max: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(rand::random_range(min..max))),
    })
}

fn sin(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.sin())),
    })
}

fn cos(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.cos())),
    })
}

fn tan(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.tan())),
    })
}

fn arcsin(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.asin())),
    })
}

fn arccos(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.acos())),
    })
}

fn arctan(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.atan())),
    })
}

fn sinh(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.sinh())),
    })
}

fn cosh(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.cosh())),
    })
}

fn clamp(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: f64, min: f64, max: f64);
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(value.clamp(min, max))),
    })
}

fn is_equal(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => lhs: f64, rhs: f64);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::context::Context;
    use crate::context::argument::Argument;
    use tucana::shared::{Value, value::Kind};

    // ---- helpers: Arguments ----
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

    // ---- helpers: extractors ----
    fn expect_num(sig: Signal) -> f64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => n,
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

    // dummy runner for handlers that accept `run: &mut dyn FnMut(i64) -> Signal`
    fn dummy_run(_: i64) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    #[test]
    fn test_add_and_multiply() {
        let mut ctx = Context::default();
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
    fn test_substract_and_divide() {
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

        let mut run = dummy_run;
        let r = expect_num(random(&[a_num(1.0), a_num(10.0)], &mut ctx, &mut run));
        assert!(r >= 1.0 && r < 10.0);
    }

    #[test]
    fn test_trig_and_hyperbolic() {
        let mut ctx = Context::default();

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
        let mut ctx = Context::default();

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
