//! Date standard-library handlers.
//!
//! `DATE` is a `NUMBER` holding Unix-epoch microseconds (UTC), matching
//! `crate::time::now_unix_micros`. Keeping the on-wire representation a plain
//! number (rather than a formatted string) means dates stay comparable/sortable
//! like any other number and avoids re-parsing on every handler call.

use crate::handler::argument::Argument;
use crate::handler::macros::{args, no_args};
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::time::now_unix_micros;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::value_from_i64;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use tucana::shared::{Value, value::Kind};

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("std::date::now", now, 0),
    FunctionRegistration::eager("std::date::from", from, 6),
    FunctionRegistration::eager("std::date::from_text", from_text, 1),
    FunctionRegistration::eager("std::date::from_unix", from_unix, 1),
    FunctionRegistration::eager("std::date::format", format, 2),
];

fn fail(message: impl Into<String>) -> Signal {
    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "InvalidArgumentRuntimeError",
        message,
    ))
}

/// `MONTH` is the string enum `'JAN' | 'FEB' | ... | 'DEC'`, not a number.
fn month_from_code(code: &str) -> Option<u32> {
    let month = match code {
        "JAN" => 1,
        "FEB" => 2,
        "MAR" => 3,
        "APR" => 4,
        "MAY" => 5,
        "JUN" => 6,
        "JUL" => 7,
        "AUG" => 8,
        "SEP" => 9,
        "OCT" => 10,
        "NOV" => 11,
        "DEC" => 12,
        _ => return None,
    };
    Some(month)
}

fn now(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_i64(now_unix_micros()))
}

fn from(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => year: i64, month: String, day: i64, hour: i64, minute: i64, second: i64);

    let Some(month) = month_from_code(&month) else {
        return fail(format!(
            "Invalid month code '{}', expected one of JAN..DEC",
            month
        ));
    };
    let Ok(year) = i32::try_from(year) else {
        return fail(format!("Year out of range: {}", year));
    };
    let (Ok(day), Ok(hour), Ok(minute), Ok(second)) = (
        u32::try_from(day),
        u32::try_from(hour),
        u32::try_from(minute),
        u32::try_from(second),
    ) else {
        return fail("day, hour, minute and second must be non-negative");
    };

    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return fail(format!(
            "Invalid date: year={}, month={}, day={}",
            year, month, day
        ));
    };
    let Some(time) = NaiveTime::from_hms_opt(hour, minute, second) else {
        return fail(format!(
            "Invalid time: hour={}, minute={}, second={}",
            hour, minute, second
        ));
    };

    Signal::Success(value_from_i64(
        date.and_time(time).and_utc().timestamp_micros(),
    ))
}

fn from_text(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    match DateTime::parse_from_rfc3339(&value) {
        Ok(parsed) => Signal::Success(value_from_i64(
            parsed.with_timezone(&Utc).timestamp_micros(),
        )),
        Err(err) => fail(format!(
            "Failed to parse date from text '{}': {}",
            value, err
        )),
    }
}

fn from_unix(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: f64);
    // Saturating float->int cast: never panics/fails, matching `throwsError: false`.
    let micros = (value * 1_000_000.0) as i64;
    Signal::Success(value_from_i64(micros))
}

/// Translates the definitions' custom pattern tokens into `chrono` strftime specifiers.
const PATTERN_TOKENS: &[(&str, &str)] = &[
    ("YYYY", "%Y"),
    ("YY", "%y"),
    ("MM", "%m"),
    ("DD", "%d"),
    ("HH", "%H"),
    ("hh", "%I"),
    ("mm", "%M"),
    ("ss", "%S"),
];

fn translate_pattern(pattern: &str) -> Result<String, String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            match PATTERN_TOKENS.iter().find(|(t, _)| *t == token) {
                Some((_, spec)) => out.push_str(spec),
                None => return Err(format!("Unknown date format token '{}'", token)),
            }
        } else if chars[i] == '%' {
            out.push_str("%%");
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    Ok(out)
}

fn format(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => date: i64, pattern: String);

    let Some(date) = DateTime::<Utc>::from_timestamp_micros(date) else {
        return fail(format!("Date is out of the representable range: {}", date));
    };
    let strftime_pattern = match translate_pattern(&pattern) {
        Ok(pattern) => pattern,
        Err(message) => return fail(message),
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(
            date.format(&strftime_pattern).to_string(),
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::number_to_i64_lossy;

    fn a_num(n: i64) -> Argument {
        Argument::Eval(value_from_i64(n))
    }
    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }

    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Stop
    }

    fn expect_num(sig: Signal) -> i64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_i64_lossy(&n).unwrap_or_default(),
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

    #[test]
    fn test_now_returns_current_micros() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let before = now_unix_micros();
        let micros = expect_num(now(&[], &mut ctx, &mut run));
        let after = now_unix_micros();
        assert!(micros >= before && micros <= after);
    }

    #[test]
    fn test_from_builds_expected_timestamp_and_rejects_invalid_input() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let micros = expect_num(from(
            &[
                a_num(2026),
                a_str("MAR"),
                a_num(15),
                a_num(10),
                a_num(30),
                a_num(0),
            ],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(
            DateTime::<Utc>::from_timestamp_micros(micros)
                .unwrap_or_default()
                .to_rfc3339(),
            "2026-03-15T10:30:00+00:00"
        );

        // invalid month code
        let mut run = dummy_run;
        match from(
            &[
                a_num(2026),
                a_str("MARCH"),
                a_num(15),
                a_num(10),
                a_num(30),
                a_num(0),
            ],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for invalid month, got {:?}", s),
        }

        // invalid day for month
        let mut run = dummy_run;
        match from(
            &[
                a_num(2026),
                a_str("FEB"),
                a_num(30),
                a_num(0),
                a_num(0),
                a_num(0),
            ],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for invalid day, got {:?}", s),
        }
    }

    #[test]
    fn test_from_text_success_and_errors() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let micros = expect_num(from_text(
            &[a_str("2026-07-28T10:15:30Z")],
            &mut ctx,
            &mut run,
        ));
        assert_eq!(
            DateTime::<Utc>::from_timestamp_micros(micros)
                .unwrap_or_default()
                .to_rfc3339(),
            "2026-07-28T10:15:30+00:00"
        );

        let mut run = dummy_run;
        match from_text(&[a_str("not a date")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for unparseable date, got {:?}", s),
        }
    }

    #[test]
    fn test_from_unix_converts_seconds_to_micros_and_never_fails() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(from_unix(
                &[Argument::Eval(value_from_i64(0))],
                &mut ctx,
                &mut run
            )),
            0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(from_unix(
                &[Argument::Eval(crate::value::value_from_f64(1.5))],
                &mut ctx,
                &mut run
            )),
            1_500_000
        );
    }

    #[test]
    fn test_format_translates_tokens_and_rejects_unknown_ones() {
        let mut ctx = ValueStore::default();
        let micros = NaiveDate::from_ymd_opt(2026, 3, 5)
            .unwrap_or_default()
            .and_time(NaiveTime::from_hms_opt(9, 5, 3).unwrap_or_default())
            .and_utc()
            .timestamp_micros();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(format(
                &[a_num(micros), a_str("YYYY-MM-DD")],
                &mut ctx,
                &mut run
            )),
            "2026-03-05"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(format(
                &[a_num(micros), a_str("HH:mm:ss")],
                &mut ctx,
                &mut run
            )),
            "09:05:03"
        );

        let mut run = dummy_run;
        match format(&[a_num(micros), a_str("YYYY-XX-DD")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for unknown token, got {:?}", s),
        }
    }
}
