//! Date standard-library handlers.
//!
//! `DATE` is a `NUMBER` holding Unix-epoch microseconds (UTC), matching
//! `crate::time::now_unix_micros`. Keeping the on-wire representation a plain
//! number (rather than a formatted string) means dates stay comparable/sortable
//! like any other number and avoids re-parsing on every handler call.

use crate::handler::argument::Argument;
use crate::handler::macros::{args, no_args};
use crate::runtime::execution::value_store::ValueStore;
use crate::time::now_unix_micros;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::value_from_i64;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use tucana::shared::{Value, value::Kind};

// This module has no `definitions/taurus-date/*.json` counterpart to port
// from (it was implemented after the JSON tree stopped being kept in sync --
// exactly the drift this self-registration replaces). The metadata below is
// authored fresh, following the same conventions as the migrated modules.
taurus_macros::module! {
    identifier = "taurus-date",
    name(en_US = "Date"),
    description(en_US = "Work with Dates."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:calendar",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "DATE",
    module = "taurus-date",
    name(en_US = "Date"),
    display_message(en_US = "Date"),
    alias(en_US = "date;time;timestamp;datetime"),
    type_string = "NUMBER",
    linked_data_type_identifiers = ["NUMBER"],
}

taurus_macros::data_type! {
    identifier = "MONTH",
    module = "taurus-date",
    name(en_US = "Month"),
    display_message(en_US = "Month"),
    alias(en_US = "month"),
    type_string = "'JAN' | 'FEB' | 'MAR' | 'APR' | 'MAY' | 'JUN' | 'JUL' | 'AUG' | 'SEP' | 'OCT' | 'NOV' | 'DEC'",
}

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

#[taurus_macros::runtime_function(
    identifier = "std::date::now",
    module = "taurus-date",
    signature = "(): DATE",
    name(en_US = "Now"),
    description(en_US = "Returns the current date and time."),
    display_message(en_US = "Now"),
    alias(en_US = "now;current;today;date;time;std"),
    display_icon = "tabler:calendar",
    linked_data_type_identifiers = ["DATE"],
)]
fn now(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    no_args!(args);
    Signal::Success(value_from_i64(now_unix_micros()))
}

#[taurus_macros::runtime_function(
    identifier = "std::date::from",
    module = "taurus-date",
    signature = "(year: NUMBER, month: MONTH, day: NUMBER, hour: NUMBER, minute: NUMBER, second: NUMBER): DATE",
    name(en_US = "Date from Parts"),
    description(en_US = "Builds a date from its year, month, day, hour, minute and second components."),
    display_message(en_US = "Date from ${year}-${month}-${day} ${hour}:${minute}:${second}"),
    alias(en_US = "from;build;construct;date;time;std"),
    display_icon = "tabler:calendar",
    linked_data_type_identifiers = ["DATE", "MONTH", "NUMBER"],
    throws_error,
)]
#[parameter(
    runtime_name = "year",
    name(en_US = "Year"),
    description(en_US = "The year component of the date.")
)]
#[parameter(
    runtime_name = "month",
    name(en_US = "Month"),
    description(en_US = "The month component of the date, as a three-letter code (JAN..DEC).")
)]
#[parameter(
    runtime_name = "day",
    name(en_US = "Day"),
    description(en_US = "The day-of-month component of the date.")
)]
#[parameter(
    runtime_name = "hour",
    name(en_US = "Hour"),
    description(en_US = "The hour component of the time.")
)]
#[parameter(
    runtime_name = "minute",
    name(en_US = "Minute"),
    description(en_US = "The minute component of the time.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second"),
    description(en_US = "The second component of the time.")
)]
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

#[taurus_macros::runtime_function(
    identifier = "std::date::from_text",
    module = "taurus-date",
    signature = "(value: TEXT): DATE",
    name(en_US = "Date from Text"),
    description(en_US = "Parses an RFC 3339 formatted text into a date."),
    display_message(en_US = "Date from ${value}"),
    alias(en_US = "from text;parse;rfc3339;date;time;std"),
    display_icon = "tabler:calendar",
    linked_data_type_identifiers = ["DATE", "TEXT"],
    throws_error,
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The RFC 3339 formatted text to parse.")
)]
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

#[taurus_macros::runtime_function(
    identifier = "std::date::from_unix",
    module = "taurus-date",
    signature = "(value: NUMBER): DATE",
    name(en_US = "Date from Unix Time"),
    description(en_US = "Converts a Unix timestamp, in seconds, to a date."),
    display_message(en_US = "Date from ${value}"),
    alias(en_US = "from unix;unix time;epoch;date;time;std"),
    display_icon = "tabler:calendar",
    linked_data_type_identifiers = ["DATE", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The Unix timestamp, in seconds, to convert.")
)]
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

#[taurus_macros::runtime_function(
    identifier = "std::date::format",
    module = "taurus-date",
    signature = "(date: DATE, pattern: TEXT): TEXT",
    name(en_US = "Format Date"),
    description(en_US = "Formats a date as text using a YYYY/MM/DD/HH/mm/ss pattern."),
    display_message(en_US = "Format ${date} as ${pattern}"),
    alias(en_US = "format;pattern;strftime;date;time;std"),
    display_icon = "tabler:calendar",
    linked_data_type_identifiers = ["DATE", "TEXT"],
    throws_error,
)]
#[parameter(
    runtime_name = "date",
    name(en_US = "Date"),
    description(en_US = "The date to format.")
)]
#[parameter(
    runtime_name = "pattern",
    name(en_US = "Pattern"),
    description(en_US = "The pattern to format the date with, e.g. \"YYYY-MM-DD HH:mm:ss\".")
)]
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
