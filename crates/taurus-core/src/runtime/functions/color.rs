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
    type_string = "{ hue: number; saturation: number; lightness: number; alpha?: number }",
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

/// Reads an optional numeric field, falling back to `default` when the field is absent or null.
fn field_f64_or(fields: &Struct, key: &str, default: f64) -> Result<f64, Signal> {
    match fields.fields.get(key).and_then(|v| v.kind.as_ref()) {
        Some(Kind::NumberValue(n)) => {
            number_to_f64(n).ok_or_else(|| fail(format!("Field '{key}' is not a valid number")))
        }
        Some(Kind::NullValue(_)) | None => Ok(default),
        _ => Err(fail(format!("Field '{key}' is not a valid number"))),
    }
}

/// Reads an optional alpha argument, defaulting to fully opaque (1.0) when omitted.
fn alpha_from_value(value: &Value) -> Result<f64, Signal> {
    let alpha = match value.kind.as_ref() {
        Some(Kind::NumberValue(n)) => {
            number_to_f64(n).ok_or_else(|| fail("Alpha is not a valid number"))?
        }
        Some(Kind::NullValue(_)) | None => return Ok(1.0),
        _ => return Err(fail("Alpha must be a number or undefined")),
    };
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail("Alpha must be between 0 and 1"));
    }
    Ok(alpha)
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

/// Converts RGB (0..=255) + optional alpha (0..=1, defaults to 1.0 when omitted) into HSLA
/// (hue 0..360, saturation/lightness 0..100, alpha 0..1).
fn rgb_to_hsl(red: f64, green: f64, blue: f64, alpha: Option<f64>) -> (f64, f64, f64, f64) {
    let alpha = alpha.unwrap_or(1.0);
    let r = red / 255.0;
    let g = green / 255.0;
    let b = blue / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let lightness = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, lightness * 100.0, alpha);
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

    (hue, saturation * 100.0, lightness * 100.0, alpha)
}

/// Converts HSLA (hue 0..360, saturation/lightness 0..100) + optional alpha (0..=1, defaults to
/// 1.0 when omitted) into RGBA (0..=255, alpha 0..=1).
fn hsl_to_rgb(
    hue: f64,
    saturation: f64,
    lightness: f64,
    alpha: Option<f64>,
) -> (f64, f64, f64, f64) {
    let alpha = alpha.unwrap_or(1.0);
    let s = saturation / 100.0;
    let l = lightness / 100.0;

    if s == 0.0 {
        let v = l * 255.0;
        return (v, v, v, alpha);
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

    ((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0, alpha)
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

    let (hue, saturation, lightness, alpha) =
        rgb_to_hsl(red as f64, green as f64, blue as f64, Some(alpha));
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
    let alpha = match field_f64_or(&value, "alpha", 1.0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    let (red, green, blue, alpha) = hsl_to_rgb(hue, saturation, lightness, Some(alpha));
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
    signature = "(red: NUMBER, green: NUMBER, blue: NUMBER, alpha?: NUMBER): COLOR",
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
    description(en_US = "The opacity, from 0 to 1. Defaults to 1 (fully opaque) when omitted.")
)]
fn from_rgb(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => red: f64, green: f64, blue: f64, alpha: Value);
    let alpha = match alpha_from_value(&alpha) {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    if ![red, green, blue].iter().all(|c| (0.0..=255.0).contains(c)) {
        return fail("Red, green and blue must be between 0 and 255");
    }

    let (hue, saturation, lightness, alpha) = rgb_to_hsl(red, green, blue, Some(alpha));
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
    let alpha = match field_f64_or(&value, "alpha", 1.0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    let (red, green, blue, alpha) = hsl_to_rgb(hue, saturation, lightness, Some(alpha));

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
    identifier = "std::color::from_hsl",
    module = "taurus-color",
    signature = "(hue: NUMBER, saturation: NUMBER, lightness: NUMBER, alpha?: NUMBER): COLOR",
    name(en_US = "Color from HSL"),
    description(
        en_US = "Builds a color from hue (0-360), saturation and lightness (0-100) and an alpha channel (0-1)."
    ),
    display_message(en_US = "Color from ${hue}, ${saturation}, ${lightness}, ${alpha}"),
    alias(en_US = "from hsl;build;convert;color;colour;std;from"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["NUMBER", "COLOR"],
)]
#[parameter(
    runtime_name = "hue",
    name(en_US = "Hue"),
    description(en_US = "The hue, from 0 to 360.")
)]
#[parameter(
    runtime_name = "saturation",
    name(en_US = "Saturation"),
    description(en_US = "The saturation, from 0 to 100.")
)]
#[parameter(
    runtime_name = "lightness",
    name(en_US = "Lightness"),
    description(en_US = "The lightness, from 0 to 100.")
)]
#[parameter(
    runtime_name = "alpha",
    name(en_US = "Alpha"),
    description(en_US = "The opacity, from 0 to 1. Defaults to 1 (fully opaque) when omitted.")
)]
fn from_hsl(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => hue: f64, saturation: f64, lightness: f64, alpha: Value);
    let alpha = match alpha_from_value(&alpha) {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    if !(0.0..=360.0).contains(&hue) {
        return fail("Hue must be between 0 and 360");
    }
    if !(0.0..=100.0).contains(&saturation) || !(0.0..=100.0).contains(&lightness) {
        return fail("Saturation and lightness must be between 0 and 100");
    }

    Signal::Success(color_from_hsla(hue, saturation, lightness, alpha))
}

#[taurus_macros::runtime_function(
    identifier = "std::color::as_hsl",
    module = "taurus-color",
    signature = "(value: COLOR): { hue: number; saturation: number; lightness: number; alpha: number }",
    name(en_US = "Color as HSL"),
    description(
        en_US = "Converts a color to its hue (0-360), saturation and lightness (0-100) and alpha (0-1) channels."
    ),
    display_message(en_US = "HSL of ${value}"),
    alias(en_US = "to hsl;convert;color;colour;std;as"),
    display_icon = "tabler:palette",
    linked_data_type_identifiers = ["COLOR", "OBJECT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The color to convert to HSL.")
)]
fn as_hsl(
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
    let alpha = match field_f64_or(&value, "alpha", 1.0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };

    let mut fields = HashMap::new();
    fields.insert("hue".to_string(), value_from_f64(hue));
    fields.insert("saturation".to_string(), value_from_f64(saturation));
    fields.insert("lightness".to_string(), value_from_f64(lightness));
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

    for key in ["hue", "saturation", "lightness"] {
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

    let lhs_alpha = match field_f64_or(&first, "alpha", 1.0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    let rhs_alpha = match field_f64_or(&second, "alpha", 1.0) {
        Ok(v) => v,
        Err(sig) => return sig,
    };
    if lhs_alpha != rhs_alpha {
        return Signal::Success(Value {
            kind: Some(Kind::BoolValue(false)),
        });
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
    fn a_null() -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }
    fn a_color_no_alpha(h: f64, s: f64, l: f64) -> Argument {
        let mut fields = HashMap::new();
        fields.insert("hue".to_string(), value_from_f64(h));
        fields.insert("saturation".to_string(), value_from_f64(s));
        fields.insert("lightness".to_string(), value_from_f64(l));
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(TcStruct { fields })),
        })
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
    fn test_from_rgb_alpha_defaults_to_opaque() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_rgb(
            &[a_num(0.0), a_num(255.0), a_num(0.0), a_null()],
            &mut ctx,
            &mut run,
        ));
        assert!((field(&color, "alpha") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_as_rgb_missing_alpha_defaults_to_opaque() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let rgb = expect_struct(as_rgb(
            &[a_color_no_alpha(120.0, 100.0, 50.0)],
            &mut ctx,
            &mut run,
        ));
        assert!((field(&rgb, "alpha") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_hsl_and_as_hsl_roundtrip() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_hsl(
            &[a_num(120.0), a_num(50.0), a_num(40.0), a_num(0.8)],
            &mut ctx,
            &mut run,
        ));
        let hsl = expect_struct(as_hsl(
            &[Argument::Eval(Value {
                kind: Some(Kind::StructValue(color)),
            })],
            &mut ctx,
            &mut run,
        ));

        assert!((field(&hsl, "hue") - 120.0).abs() < 1e-6);
        assert!((field(&hsl, "saturation") - 50.0).abs() < 1e-6);
        assert!((field(&hsl, "lightness") - 40.0).abs() < 1e-6);
        assert!((field(&hsl, "alpha") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_from_hsl_out_of_range() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        match from_hsl(
            &[a_num(400.0), a_num(0.0), a_num(0.0), a_num(1.0)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        match from_hsl(
            &[a_num(0.0), a_num(150.0), a_num(0.0), a_num(1.0)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        match from_hsl(
            &[a_num(0.0), a_num(0.0), a_num(0.0), a_num(2.0)],
            &mut ctx,
            &mut run,
        ) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
    }

    #[test]
    fn test_from_hsl_alpha_defaults_to_opaque() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let color = expect_struct(from_hsl(
            &[a_num(120.0), a_num(50.0), a_num(40.0), a_null()],
            &mut ctx,
            &mut run,
        ));
        assert!((field(&color, "alpha") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_as_hsl_missing_alpha_defaults_to_opaque() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        let hsl = expect_struct(as_hsl(
            &[a_color_no_alpha(120.0, 50.0, 40.0)],
            &mut ctx,
            &mut run,
        ));
        assert!((field(&hsl, "alpha") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_equal_missing_alpha_treated_as_opaque() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        assert!(expect_bool(is_equal(
            &[
                a_color_no_alpha(10.0, 20.0, 30.0),
                a_color(10.0, 20.0, 30.0, 1.0)
            ],
            &mut ctx,
            &mut run
        )));
        assert!(!expect_bool(is_equal(
            &[
                a_color_no_alpha(10.0, 20.0, 30.0),
                a_color(10.0, 20.0, 30.0, 0.5)
            ],
            &mut ctx,
            &mut run
        )));
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
