//! Color standard-library handlers.
//!
//! `COLOR` is an object of HSLA components (hue, saturation, lightness,
//! alpha), matching `type_string` below.

// This module had no `definitions/taurus-color/*.json` counterpart in this
// repo (it was implemented after the JSON tree stopped being kept in sync --
// exactly the drift this self-registration replaces). The metadata below is
// transcribed from Aquila's live definitions instead.
taurus_macros::module! {
    identifier = "taurus-color",
    name(en_US = "Color"),
    description(en_US = "Work with Colors."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:palette",
    version = "0.0.34",
}

taurus_macros::data_type! {
    identifier = "COLOR",
    module = "taurus-color",
    name(en_US = "Color"),
    display_message(en_US = "Color"),
    alias(en_US = "color;colour;grb;hsv;hsl;hex"),
    type_string = "{ hue: number; saturation: number; lightness: number; alpha: number }",
}

use std::collections::HashMap;

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::{number_to_f64, value_from_f64};
use tucana::shared::{Struct, Value, value::Kind};

fn fail(message: impl Into<String>) -> Signal {
    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "InvalidArgumentRuntimeError",
        message.into(),
    ))
}

fn field_f64(fields: &Struct, key: &str) -> Result<f64, Signal> {
    match fields.fields.get(key).and_then(|v| v.kind.as_ref()) {
        Some(Kind::NumberValue(n)) => {
            number_to_f64(n).ok_or_else(|| fail(format!("Field '{key}' is not a valid number")))
        }
        _ => Err(fail(format!("Color is missing numeric field '{key}'"))),
    }
}

fn color_from_hsla(hue: f64, saturation: f64, lightness: f64, alpha: f64) -> Value {
    let mut fields = HashMap::new();
    fields.insert("hue".to_string(), value_from_f64(hue));
    fields.insert("saturation".to_string(), value_from_f64(saturation));
    fields.insert("lightness".to_string(), value_from_f64(lightness));
    fields.insert("alpha".to_string(), value_from_f64(alpha));
    Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    }
}

/// Converts RGB (0..=255) + alpha (0..=1) into HSLA (hue 0..360, saturation/lightness 0..100, alpha 0..1).
fn rgb_to_hsl(red: f64, green: f64, blue: f64) -> (f64, f64, f64) {
    let r = red / 255.0;
    let g = green / 255.0;
    let b = blue / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, lightness * 100.0);
    }

    let saturation = if lightness <= 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let hue = if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    let mut hue = hue * 60.0;
    if hue < 0.0 {
        hue += 360.0;
    }

    (hue, saturation * 100.0, lightness * 100.0)
}

/// Converts HSLA (hue 0..360, saturation/lightness 0..100) into RGB (0..=255).
fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> (f64, f64, f64) {
    let s = saturation / 100.0;
    let l = lightness / 100.0;

    if s == 0.0 {
        let v = l * 255.0;
        return (v, v, v);
    }

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h = hue / 60.0;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    ((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0)
}

fn parse_hex_byte(hex: &str, at: usize) -> Result<u8, Signal> {
    hex.get(at..at + 2)
        .and_then(|s| u8::from_str_radix(s, 16).ok())
        .ok_or_else(|| fail(format!("Invalid hex color: {:?}", hex)))
}

#[taurus_macros::runtime_function(
    identifier = "std::color::from_hex",
    module = "taurus-color",
    signature = "(value: TEXT): COLOR",
    name(en_US = "Color from Hex"),
    description(
        en_US = "Parses a `#RRGGBB` or `#RRGGBBAA` hex string into a color."
    ),
    display_message(en_US = "Color from ${value}"),
    alias(en_US = "from hex;parse;convert;color;colour;std;from"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["TEXT", "COLOR"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The hex text to parse, e.g. `#FF00FF` or `#FF00FFAA`.")
)]
fn from_hex(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    let hex = value.strip_prefix('#').unwrap_or(&value);

    if hex.len() != 6 && hex.len() != 8 {
        return fail(format!("Invalid hex color: {:?}", value));
    }

    let red = match parse_hex_byte(hex, 0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let green = match parse_hex_byte(hex, 2) {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let blue = match parse_hex_byte(hex, 4) {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let alpha = if hex.len() == 8 {
        match parse_hex_byte(hex, 6) {
            Ok(v) => v as f64 / 255.0,
            Err(sig) => return sig,
        }
    } else {
        1.0
    };

    let (hue, saturation, lightness) = rgb_to_hsl(red as f64, green as f64, blue as f64);
    Signal::Success(color_from_hsla(hue, saturation, lightness, alpha))
}

#[taurus_macros::runtime_function(
    identifier = "std::color::as_hex",
    module = "taurus-color",
    signature = "(value: COLOR): TEXT",
    name(en_US = "Color as Hex"),
    description(
        en_US = "Formats a color as a `#RRGGBB` hex string, or `#RRGGBBAA` when alpha is not fully opaque."
    ),
    display_message(en_US = "Hex of ${value}"),
    alias(en_US = "to hex;format;convert;color;colour;std;as"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["COLOR", "TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The color to format as hex.")
)]
fn as_hex(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: Struct);
    let hue = match field_f64(&value, "hue") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let saturation = match field_f64(&value, "saturation") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let lightness = match field_f64(&value, "lightness") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let alpha = match field_f64(&value, "alpha") {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    let (red, green, blue) = hsl_to_rgb(hue, saturation, lightness);
    let hex = if alpha >= 1.0 {
        format!(
            "#{:02X}{:02X}{:02X}",
            red.round() as u8,
            green.round() as u8,
            blue.round() as u8
        )
    } else {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            red.round() as u8,
            green.round() as u8,
            blue.round() as u8,
            (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
        )
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(hex)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::color::from_rgb",
    module = "taurus-color",
    signature = "(red: NUMBER, green: NUMBER, blue: NUMBER, alpha: NUMBER): COLOR",
    name(en_US = "Color from RGB"),
    description(
        en_US = "Builds a color from red, green and blue channels (0-255) and an alpha channel (0-1)."
    ),
    display_message(en_US = "Color from ${red}, ${green}, ${blue}, ${alpha}"),
    alias(en_US = "from rgb;build;convert;color;colour;std;from"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["NUMBER", "COLOR"],
)]
#[parameter(
    runtime_name = "red",
    name(en_US = "Red"),
    description(en_US = "The red channel, from 0 to 255.")
)]
#[parameter(
    runtime_name = "green",
    name(en_US = "Green"),
    description(en_US = "The green channel, from 0 to 255.")
)]
#[parameter(
    runtime_name = "blue",
    name(en_US = "Blue"),
    description(en_US = "The blue channel, from 0 to 255.")
)]
#[parameter(
    runtime_name = "alpha",
    name(en_US = "Alpha"),
    description(en_US = "The opacity, from 0 to 1.")
)]
fn from_rgb(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => red: f64, green: f64, blue: f64, alpha: f64);

    if ![red, green, blue].iter().all(|c| (0.0..=255.0).contains(c)) {
        return fail("Red, green and blue must be between 0 and 255");
    }
    if !(0.0..=1.0).contains(&alpha) {
        return fail("Alpha must be between 0 and 1");
    }

    let (hue, saturation, lightness) = rgb_to_hsl(red, green, blue);
    Signal::Success(color_from_hsla(hue, saturation, lightness, alpha))
}

#[taurus_macros::runtime_function(
    identifier = "std::color::as_rgb",
    module = "taurus-color",
    signature = "(value: COLOR): { red: number; green: number; blue: number; alpha: number }",
    name(en_US = "Color as RGB"),
    description(
        en_US = "Converts a color to its red, green, blue (0-255) and alpha (0-1) channels."
    ),
    display_message(en_US = "RGB of ${value}"),
    alias(en_US = "to rgb;convert;color;colour;std;as"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["COLOR", "OBJECT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The color to convert to RGB.")
)]
fn as_rgb(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: Struct);
    let hue = match field_f64(&value, "hue") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let saturation = match field_f64(&value, "saturation") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let lightness = match field_f64(&value, "lightness") {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let alpha = match field_f64(&value, "alpha") {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    let (red, green, blue) = hsl_to_rgb(hue, saturation, lightness);

    let mut fields = HashMap::new();
    fields.insert("red".to_string(), value_from_f64(red.round()));
    fields.insert("green".to_string(), value_from_f64(green.round()));
    fields.insert("blue".to_string(), value_from_f64(blue.round()));
    fields.insert("alpha".to_string(), value_from_f64(alpha));

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::color::is_equal",
    module = "taurus-color",
    signature = "(first: COLOR, second: COLOR): BOOLEAN",
    name(en_US = "Is Equal"),
    description(
        en_US = "Compares two colors for equality. Returns true if all HSLA components are the same, false otherwise."
    ),
    display_message(en_US = "${first} Equals ${second}"),
    alias(en_US = "equal;equals;same;color;colour;std;is"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["COLOR", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First"),
    description(en_US = "The first color to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second"),
    description(en_US = "The second color to compare.")
)]
fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => first: Struct, second: Struct);

    for key in ["hue", "saturation", "lightness", "alpha"] {
        let lhs = match field_f64(&first, key) {
            Ok(v) => v,
            Err(sig) => return sig,
        };
        let rhs = match field_f64(&second, key) {
            Ok(v) => v,
            Err(sig) => return sig,
        };
        if lhs != rhs {
            return Signal::Success(Value {
                kind: Some(Kind::BoolValue(false)),
            });
        }
    }

    Signal::Success(Value {
        kind: Some(Kind::BoolValue(true)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::value_from_f64;
    use tucana::shared::{Struct as TcStruct, Value, value::Kind};

    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }
    fn a_num(n: f64) -> Argument {
        Argument::Eval(value_from_f64(n))
    }
    fn a_color(h: f64, s: f64, l: f64, a: f64) -> Argument {
        let mut fields = HashMap::new();
        fields.insert("hue".to_string(), value_from_f64(h));
        fields.insert("saturation".to_string(), value_from_f64(s));
        fields.insert("lightness".to_string(), value_from_f64(l));
        fields.insert("alpha".to_string(), value_from_f64(a));
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(TcStruct { fields })),
        })
    }

    fn expect_str(sig: Signal) -> String {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::StringValue(s)),
            }) => s,
            other => panic!("Expected StringValue, got {:?}", other),
        }
    }
    fn expect_struct(sig: Signal) -> TcStruct {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::StructValue(s)),
            }) => s,
            other => panic!("Expected StructValue, got {:?}", other),
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
    fn field(s: &TcStruct, key: &str) -> f64 {
        field_f64(s, key).map_err(|_| ()).unwrap_or_else(|_| {
            panic!("missing field {key}");
        })
    }

    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    #[test]
    fn test_from_hex_rgb_roundtrip() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_hex(&[a_str("#FF0000")], &mut ctx, &mut run));
        assert!((field(&color, "hue") - 0.0).abs() < 1e-6);
        assert!((field(&color, "saturation") - 100.0).abs() < 1e-6);
        assert!((field(&color, "lightness") - 50.0).abs() < 1e-6);
        assert!((field(&color, "alpha") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_hex_with_alpha_and_no_hash() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_hex(&[a_str("00FF0080")], &mut ctx, &mut run));
        assert!((field(&color, "alpha") - (0x80 as f64 / 255.0)).abs() < 1e-6);
    }

    #[test]
    fn test_from_hex_invalid() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match from_hex(&[a_str("#ZZZZZZ")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        match from_hex(&[a_str("#FFF")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
    }

    #[test]
    fn test_as_hex_roundtrip() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        assert_eq!(
            expect_str(as_hex(
                &[a_color(0.0, 100.0, 50.0, 1.0)],
                &mut ctx,
                &mut run
            )),
            "#FF0000"
        );
        assert_eq!(
            expect_str(as_hex(
                &[a_color(0.0, 100.0, 50.0, 0.5)],
                &mut ctx,
                &mut run
            )),
            "#FF000080"
        );
    }

    #[test]
    fn test_from_rgb_and_as_rgb_roundtrip() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_rgb(
            &[a_num(0.0), a_num(255.0), a_num(0.0), a_num(1.0)],
            &mut ctx,
            &mut run,
        ));
        let rgb = expect_struct(as_rgb(
            &[Argument::Eval(Value {
                kind: Some(Kind::StructValue(color)),
            })],
            &mut ctx,
            &mut run,
        ));

        assert!((field(&rgb, "red") - 0.0).abs() < 1.0);
        assert!((field(&rgb, "green") - 255.0).abs() < 1.0);
        assert!((field(&rgb, "blue") - 0.0).abs() < 1.0);
    }

    #[test]
    fn test_from_rgb_out_of_range() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match from_rgb(
            &[a_num(300.0), a_num(0.0), a_num(0.0), a_num(1.0)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        match from_rgb(
            &[a_num(0.0), a_num(0.0), a_num(0.0), a_num(2.0)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
    }

    #[test]
    fn test_is_equal() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        assert!(expect_bool(is_equal(
            &[
                a_color(10.0, 20.0, 30.0, 1.0),
                a_color(10.0, 20.0, 30.0, 1.0)
            ],
            &mut ctx,
            &mut run
        )));
        assert!(!expect_bool(is_equal(
            &[
                a_color(10.0, 20.0, 30.0, 1.0),
                a_color(10.0, 20.0, 30.0, 0.5)
            ],
            &mut ctx,
            &mut run
        )));
    }
}
