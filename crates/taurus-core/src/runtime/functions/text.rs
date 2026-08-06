//! Text/string standard-library handlers.
//!
//! Index semantics are explicit per operation:
//! - character-based for access-oriented operations like `at`
//! - byte-based where direct string insertion/removal APIs are used
//!   (to preserve historical runtime behavior)

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::{number_to_f64, number_to_i64_lossy, value_from_i64};
use base64::Engine;
use tucana::shared::{ListValue, Value, value::Kind};

taurus_macros::module! {
    identifier = "taurus-text",
    name(en_US = "Text"),
    description(en_US = "Work with Text."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:abc",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "TEXT",
    module = "taurus-text",
    name(en_US = "Text"),
    display_message(en_US = "Text"),
    alias(en_US = "text;char;literal;string"),
    type_string = "string",
}

taurus_macros::data_type! {
    identifier = "TEXT_ENCODING",
    module = "taurus-text",
    name(en_US = "Text Encoding"),
    display_message(en_US = "Text Encoding"),
    alias(en_US = "text;encoding;base64"),
    type_string = "'BASE64'",
}

fn arg_err<S: Into<String>>(msg: S) -> Signal {
    Signal::Failure(RuntimeError::new(
        "T-STD-00001",
        "InvalidArgumentRuntimeError",
        msg.into(),
    ))
}

#[taurus_macros::runtime_function(
    identifier = "std::text::as_bytes",
    module = "taurus-text",
    signature = "(value: TEXT): LIST<NUMBER>",
    name(en_US = "Text As Bytes"),
    description(en_US = "Converts a text into a list of byte values."),
    display_message(en_US = "${value} As Bytes"),
    alias(en_US = "as_bytes;text;string;std;as;bytes"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "Converts the input text into a list of byte values.")
)]
fn as_bytes(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let bytes: Vec<Value> = value
        .as_bytes()
        .iter()
        .map(|b| value_from_i64(*b as i64))
        .collect();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: bytes })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::byte_size",
    module = "taurus-text",
    signature = "(value: TEXT): NUMBER",
    name(en_US = "Byte Size"),
    description(en_US = "Computes the size in bytes of the provided text."),
    display_message(en_US = "Byte-Size of ${value}"),
    alias(en_US = "byte_size;text;string;std;byte;size"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "The text whose byte size is to be calculated.")
)]
fn byte_size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    Signal::Success(value_from_i64(value.len() as i64))
}

#[taurus_macros::runtime_function(
    identifier = "std::text::capitalize",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Capitalize"),
    description(en_US = "Converts the first character of the text to uppercase and leaves the rest unchanged."),
    display_message(en_US = "Capitalize ${value}"),
    alias(en_US = "capitalize;title case;upper first;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "Capitalizes the first letter of the input text.")
)]
fn capitalize(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let capitalized = value
        .split(' ')
        .map(|word| {
            if word.is_empty() {
                return String::from(word);
            }
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::from(word),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    Signal::Success(Value {
        kind: Some(Kind::StringValue(capitalized)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::uppercase",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Uppercase"),
    description(en_US = "Transforms all letters in the text to their uppercase equivalents."),
    display_message(en_US = "Uppercase ${value}"),
    alias(en_US = "uppercase;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "Converts all characters in the input text to uppercase.")
)]
fn uppercase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_uppercase())),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::lowercase",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Text to Lowercase"),
    description(en_US = "Returns a new text with all characters converted to lowercase."),
    display_message(en_US = "${value} to Lowercase"),
    alias(en_US = "lowercase;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "Converts all characters in the input text to lowercase.")
)]
fn lowercase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.to_lowercase())),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::swapcase",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Swap Case"),
    description(en_US = "Converts uppercase letters to lowercase and lowercase letters to uppercase in the given text."),
    display_message(en_US = "Swapcase of ${value}"),
    alias(en_US = "swapcase;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(
        en_US = "Swaps the case of each letter in the input text: uppercase letters become lowercase, and vice versa."
    )
)]
fn swapcase(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let swapped = value
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().collect::<String>()
            } else if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(swapped)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::trim",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Trim Text"),
    description(en_US = "Returns a new text with all leading and trailing whitespace characters removed from the input text."),
    display_message(en_US = "Trim ${value}"),
    alias(en_US = "trim;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(
        en_US = "The input text from which leading and trailing whitespace characters will be removed."
    )
)]
fn trim(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value.trim().to_string())),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::chars",
    module = "taurus-text",
    signature = "(value: TEXT): LIST<TEXT>",
    name(en_US = "Characters"),
    description(en_US = "Creates a list where each element is a single character from the original text."),
    display_message(en_US = "Turns ${value} into a List of Characters"),
    alias(en_US = "characters;letters;split;text;string;std;chars"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "LIST"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "Splits the input text into a list of its constituent characters.")
)]
fn chars(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let list = value
        .chars()
        .map(|c| Value {
            kind: Some(Kind::StringValue(c.to_string())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: list })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::at",
    module = "taurus-text",
    signature = "(value: TEXT, index: NUMBER): TEXT",
    name(en_US = "Character at Index"),
    description(en_US = "Retrieves a single character from the input text based on the provided zero-based index."),
    display_message(en_US = "Get Character of ${value} at ${index}"),
    alias(en_US = "at;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["NUMBER", "TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text Value"),
    description(en_US = "The input text from which a character will be retrieved by index.")
)]
#[parameter(
    runtime_name = "index",
    name(en_US = "Index"),
    description(en_US = "The zero-based position of the character to extract.")
)]
fn at(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, index: tucana::shared::NumberValue);
    let index = match number_to_i64_lossy(&index) {
        Some(v) => v,
        None => return arg_err("Expected a number index"),
    };

    if index < 0 {
        return arg_err("Expected a non-negative index");
    }

    let idx = index as usize;
    match value.chars().nth(idx) {
        Some(c) => Signal::Success(Value {
            kind: Some(Kind::StringValue(c.to_string())),
        }),
        None => Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!(
                "Index {} is out of bounds for string of length {}",
                index,
                value.chars().count()
            ),
        )),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::text::append",
    module = "taurus-text",
    signature = "(value: TEXT, suffix: TEXT): TEXT",
    name(en_US = "Append Text"),
    description(en_US = "Returns a new text consisting of the original text followed by the specified suffix."),
    display_message(en_US = "Append ${suffix} at the End of ${value}"),
    alias(en_US = "append;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(en_US = "The base text that will have another text appended to its end.")
)]
#[parameter(
    runtime_name = "suffix",
    name(en_US = "Suffix"),
    description(en_US = "The text that will be appended to the end of the original text.")
)]
fn append(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, suffix: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(value + &suffix)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::prepend",
    module = "taurus-text",
    signature = "(value: TEXT, prefix: TEXT): TEXT",
    name(en_US = "Prepend Text"),
    description(en_US = "Returns a new text consisting of the specified prefix followed by the original text."),
    display_message(en_US = "Prepend ${value} with ${prefix}"),
    alias(en_US = "prepend;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(en_US = "The base text that will have another text prepended to its beginning.")
)]
#[parameter(
    runtime_name = "prefix",
    name(en_US = "Prefix"),
    description(en_US = "The text that will be added to the start of the original text.")
)]
fn prepend(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, prefix: String);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(prefix + &value)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::insert",
    module = "taurus-text",
    signature = "(value: TEXT, position: NUMBER, text: TEXT): TEXT",
    name(en_US = "Insert Text"),
    description(en_US = "Returns a new text where the provided text is inserted at the zero-based position index within the original text."),
    display_message(en_US = "Insert ${value} at ${position} into ${text}"),
    alias(en_US = "insert;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(en_US = "The original text into which another text will be inserted.")
)]
#[parameter(
    runtime_name = "position",
    name(en_US = "Position"),
    description(en_US = "Zero-based index indicating where the new text should be inserted.")
)]
#[parameter(
    runtime_name = "text",
    name(en_US = "Text to Insert"),
    description(en_US = "The text that will be inserted into the original text.")
)]
fn insert(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, position: tucana::shared::NumberValue, text: String);
    let position = match number_to_i64_lossy(&position) {
        Some(v) => v,
        None => return arg_err("Expected a number position"),
    };

    if position < 0 {
        return arg_err("Expected a non-negative position");
    }

    let pos = position as usize;
    // Byte-wise position is kept intentionally to match existing flow behavior.
    if pos > value.len() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!("Position {} exceeds byte length {}", pos, value.len()),
        ));
    }

    let mut new_value = value;
    new_value.insert_str(pos, &text);

    Signal::Success(Value {
        kind: Some(Kind::StringValue(new_value)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::length",
    module = "taurus-text",
    signature = "(value: TEXT): NUMBER",
    name(en_US = "Length"),
    description(en_US = "Returns the number of characters in the given text."),
    display_message(en_US = "Length of ${value}"),
    alias(en_US = "length;size;characters;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "Input text to determine the number of characters it contains.")
)]
fn length(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);
    Signal::Success(value_from_i64(value.chars().count() as i64))
}

#[taurus_macros::runtime_function(
    identifier = "std::text::remove",
    module = "taurus-text",
    signature = "(value: TEXT, start: NUMBER, end: NUMBER): TEXT",
    name(en_US = "Remove String"),
    description(en_US = "Removes the subtext between the specified start and end indices from the input text."),
    display_message(en_US = "Remove ${value} from ${start}"),
    alias(en_US = "remove;delete;strip;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The input text from which a subtext will be removed.")
)]
#[parameter(
    runtime_name = "start",
    name(en_US = "Start Index"),
    description(en_US = "The starting position for removing characters from the text.")
)]
#[parameter(
    runtime_name = "end",
    name(en_US = "End Index"),
    description(en_US = "The zero-based index where removal ends (exclusive).")
)]
fn remove(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, from: tucana::shared::NumberValue, to: tucana::shared::NumberValue);
    let from = match number_to_i64_lossy(&from) {
        Some(v) => v,
        None => return arg_err("Expected number 'from'"),
    };
    let to = match number_to_i64_lossy(&to) {
        Some(v) => v,
        None => return arg_err("Expected number 'to'"),
    };

    if from < 0 || to < 0 {
        return arg_err("Expected non-negative indices");
    }

    let from_u = from as usize;
    let to_u = to as usize;

    let chars = value.chars().collect::<Vec<char>>();
    if from_u > chars.len() || to_u > chars.len() {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "IndexOutOfBoundsRuntimeError",
            format!(
                "Indices [{}, {}) out of bounds for length {}",
                from_u,
                to_u,
                chars.len()
            ),
        ));
    }

    let new = chars
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i < from_u || i >= to_u)
        .map(|e| e.1)
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(new)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::replace",
    module = "taurus-text",
    signature = "(value: TEXT, oldText: TEXT, newText: TEXT): TEXT",
    name(en_US = "Replace Subtext"),
    description(en_US = "Returns a new text where every instance of the old subtext is replaced by the new subtext."),
    display_message(en_US = "Replace ${old} with ${new} Inside ${value}"),
    alias(en_US = "replace;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(
        en_US = "This is the text in which all occurrences of the old subtext will be replaced."
    )
)]
#[parameter(
    runtime_name = "oldText",
    name(en_US = "Old Subtext"),
    description(en_US = "All occurrences of this subtext in the original text will be replaced.")
)]
#[parameter(
    runtime_name = "newText",
    name(en_US = "New Subtext"),
    description(en_US = "This subtext will replace each occurrence of the old subtext.")
)]
fn replace(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, old: String, new: String);
    let replaced = value.replace(&old, &new);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::replace_first",
    module = "taurus-text",
    signature = "(value: TEXT, oldText: TEXT, newText: TEXT): TEXT",
    name(en_US = "Replace First Subtext"),
    description(en_US = "Replaces the first occurrence of a specified subtext with another subtext in the input text."),
    documentation(en_US = "Returns a new text where only the first instance of the old subtext is replaced by the new subtext."),
    display_message(en_US = "In ${value} replace first ${old} with ${new}"),
    alias(en_US = "replace_first;text;string;std;replace;first"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(
        en_US = "This text contains the subtext that will be replaced only once—the first occurrence."
    )
)]
#[parameter(
    runtime_name = "oldText",
    name(en_US = "Old Subtext"),
    description(
        en_US = "Only the first occurrence of this subtext will be replaced in the original text."
    )
)]
#[parameter(
    runtime_name = "newText",
    name(en_US = "New Subtext"),
    description(en_US = "This subtext will replace only the first occurrence of the old subtext.")
)]
fn replace_first(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, old: String, new: String);
    let replaced = value.replacen(&old, &new, 1);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::replace_last",
    module = "taurus-text",
    signature = "(value: TEXT, oldText: TEXT, newText: TEXT): TEXT",
    name(en_US = "Replace Last Text"),
    description(en_US = "Replaces the last occurrence of a specified subtext with another subtext in the input text."),
    display_message(en_US = "In ${value} replace the last ${old} with ${new}"),
    alias(en_US = "replace_last;text;string;std;replace;last"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Original Text"),
    description(
        en_US = "This text contains the subtext that will be replaced only once—the last occurrence."
    )
)]
#[parameter(
    runtime_name = "oldText",
    name(en_US = "Old Subtext"),
    description(
        en_US = "Only the last occurrence of this subtext will be replaced in the original text."
    )
)]
#[parameter(
    runtime_name = "newText",
    name(en_US = "New Subtext"),
    description(en_US = "This subtext will replace only the last occurrence of the old subtext.")
)]
fn replace_last(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, old: String, new: String);

    fn replace_last_impl(haystack: &str, needle: &str, replacement: &str) -> String {
        if let Some(pos) = haystack.rfind(needle) {
            let mut result =
                String::with_capacity(haystack.len() - needle.len() + replacement.len());
            result.push_str(&haystack[..pos]);
            result.push_str(replacement);
            result.push_str(&haystack[pos + needle.len()..]);
            result
        } else {
            haystack.to_string()
        }
    }

    let replaced = replace_last_impl(&value, &old, &new);
    Signal::Success(Value {
        kind: Some(Kind::StringValue(replaced)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::hex",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Text to Hexadecimal"),
    description(en_US = "Returns a text containing the hexadecimal values corresponding to each character of the input text."),
    display_message(en_US = "${value} to Hexadecimal"),
    alias(en_US = "hex;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Text"),
    description(en_US = "The text to be converted to its hexadecimal representation.")
)]
fn hex(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let hex = value
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(hex)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::octal",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Text to Octal"),
    description(en_US = "Converts a text into an octal representation."),
    display_message(en_US = "${value} to Octal"),
    alias(en_US = "octal;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Input Text"),
    description(en_US = "The text to be converted to its octal representation.")
)]
fn octal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let oct = value
        .as_bytes()
        .iter()
        .map(|b| format!("{:03o}", b))
        .collect::<String>();

    Signal::Success(Value {
        kind: Some(Kind::StringValue(oct)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::index_of",
    module = "taurus-text",
    signature = "(value: TEXT, substring: TEXT): NUMBER",
    name(en_US = "Index Of"),
    description(en_US = "Returns the zero-based index of the first occurrence of the subtext in the text. Returns -1 if the subtext is not found."),
    display_message(en_US = "Get Position of ${substring} Inside ${value}"),
    alias(en_US = "index_of;text;string;std;index;of"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The text to search within.")
)]
#[parameter(
    runtime_name = "substring",
    name(en_US = "Subtext"),
    description(en_US = "The subtext to find inside the text.")
)]
fn index_of(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, sub: String);

    match value.find(&sub) {
        Some(idx) => Signal::Success(value_from_i64(idx as i64)),
        None => Signal::Success(value_from_i64(-1)),
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::text::contains",
    module = "taurus-text",
    signature = "(value: TEXT, substring: TEXT): BOOLEAN",
    name(en_US = "Contains Text"),
    description(en_US = "Returns true if the subtext is found anywhere in the main text. Otherwise, returns false."),
    display_message(en_US = "Check if ${value} contains ${substring}"),
    alias(en_US = "contains;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The main text to search within.")
)]
#[parameter(
    runtime_name = "substring",
    name(en_US = "Subtext"),
    description(en_US = "The text to search for inside the main text.")
)]
fn contains(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, sub: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.contains(&sub))),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::split",
    module = "taurus-text",
    signature = "(value: TEXT, delimiter: TEXT): LIST<TEXT>",
    name(en_US = "Split"),
    description(en_US = "Returns a list of subtext obtained by splitting the input text at each occurrence of the delimiter."),
    display_message(en_US = "Splits ${value} on '${delimiter}'"),
    alias(en_US = "split;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "LIST"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The input text to be split.")
)]
#[parameter(
    runtime_name = "delimiter",
    name(en_US = "Delimiter"),
    description(en_US = "The delimiter text to split the text by.")
)]
fn split(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, delimiter: String);

    let parts = value
        .split(&delimiter)
        .map(|s| Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: parts })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::reverse",
    module = "taurus-text",
    signature = "(value: TEXT): TEXT",
    name(en_US = "Reverse Text"),
    description(en_US = "Returns a new text with the characters of the input text in reverse order."),
    display_message(en_US = "Reverse ${value}"),
    alias(en_US = "reverse;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The input text to be reversed.")
)]
fn reverse(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let reversed = value.chars().rev().collect::<String>();
    Signal::Success(Value {
        kind: Some(Kind::StringValue(reversed)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::starts_with",
    module = "taurus-text",
    signature = "(value: TEXT, prefix: TEXT): BOOLEAN",
    name(en_US = "Starts With"),
    description(en_US = "Returns true if the input text begins with the given prefix. Otherwise, returns false."),
    display_message(en_US = "Check if ${value} starts with ${prefix}"),
    alias(en_US = "text;string;std;start;with;starts"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The input text to check.")
)]
#[parameter(
    runtime_name = "prefix",
    name(en_US = "Prefix"),
    description(en_US = "The prefix to test against the input text.")
)]
fn starts_with(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, prefix: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.starts_with(&prefix))),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::ends_with",
    module = "taurus-text",
    signature = "(value: TEXT, suffix: TEXT): BOOLEAN",
    name(en_US = "Ends With"),
    description(en_US = "Returns true if the input text ends with the given suffix. Otherwise, returns false."),
    display_message(en_US = "Check if ${value} Ends With ${suffix}"),
    alias(en_US = "ends_with;text;string;std;ends;with"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The input text to check.")
)]
#[parameter(
    runtime_name = "suffix",
    name(en_US = "Suffix"),
    description(en_US = "The suffix to test against the input text.")
)]
fn ends_with(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, suffix: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(value.ends_with(&suffix))),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::to_ascii",
    module = "taurus-text",
    signature = "(value: TEXT): LIST<NUMBER>",
    name(en_US = "Text to ASCII"),
    description(en_US = "Returns a list of numbers where each number represents the ASCII code of the corresponding character in the input text."),
    display_message(en_US = "${value} To Ascii"),
    alias(en_US = "to_ascii;text;string;std;to;ascii"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "LIST", "NUMBER"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "Input text to convert to ASCII codes.")
)]
fn to_ascii(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String);

    let ascii = value
        .bytes()
        .map(|b| value_from_i64(b as i64))
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(ListValue { values: ascii })),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::from_ascii",
    module = "taurus-text",
    signature = "(value: LIST<NUMBER>): TEXT",
    name(en_US = "Text from ASCII"),
    description(en_US = "Converts a list of ASCII codes back into the corresponding text."),
    display_message(en_US = "${value} to Text"),
    alias(en_US = "from_ascii;text;string;std;from;ascii"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "NUMBER", "LIST"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "ASCII Code"),
    description(en_US = "List of ASCII numeric codes representing characters.")
)]
fn from_ascii(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    // Requires a TryFromArg impl for ListValue in your macro system.
    args!(args => list: ListValue);

    let string = list
        .values
        .iter()
        .map(|v| match v {
            Value {
                kind: Some(Kind::NumberValue(n)),
            } => match number_to_f64(n) {
                Some(n) if (0.0..=127.0).contains(&n) => Some(n as u8 as char),
                _ => None,
            },
            _ => None,
        })
        .collect::<Option<String>>();

    match string {
        Some(s) => Signal::Success(Value {
            kind: Some(Kind::StringValue(s)),
        }),
        None => arg_err("Expected a list of numbers between 0 and 127"),
    }
}

// NOTE: "encode"/"decode" currently only support base64.
#[taurus_macros::runtime_function(
    identifier = "std::text::encode",
    module = "taurus-text",
    signature = "(value: TEXT, encoding: TEXT_ENCODING): TEXT",
    name(en_US = "Encode Text"),
    description(en_US = "Transforms the given text into a representation encoded by the specified encoding scheme."),
    display_message(en_US = "Encode ${value} to ${encoding}"),
    alias(en_US = "encode;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "TEXT_ENCODING"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The text to encode.")
)]
#[parameter(
    runtime_name = "encoding",
    name(en_US = "Encoding Type"),
    description(en_US = "The encoding scheme to apply (e.g., UTF-8, Base64).")
)]
fn encode(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, encoding: String);

    let encoded = match encoding.to_lowercase().as_str() {
        "base64" => base64::prelude::BASE64_STANDARD.encode(value),
        _ => {
            return arg_err(format!("Unsupported encoding: {}", encoding));
        }
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(encoded)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::decode",
    module = "taurus-text",
    signature = "(value: TEXT, encoding: TEXT_ENCODING): TEXT",
    name(en_US = "Decode Text"),
    description(en_US = "Decodes the input text from the specified encoding format."),
    display_message(en_US = "Decode ${value} using ${encoding}"),
    alias(en_US = "decode;text;string;std"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "TEXT_ENCODING"],
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Text"),
    description(en_US = "The text to decode.")
)]
#[parameter(
    runtime_name = "encoding",
    name(en_US = "Encoding Type"),
    description(en_US = "The decoding scheme to apply (e.g. Base64).")
)]
fn decode(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: String, encoding: String);

    let decoded = match encoding.to_lowercase().as_str() {
        "base64" => match base64::prelude::BASE64_STANDARD.decode(value) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(err) => {
                    return Signal::Failure(RuntimeError::new(
                        "T-STD-00001",
                        "DecodeError",
                        format!("Failed to decode base64 bytes to UTF-8: {:?}", err),
                    ));
                }
            },
            Err(err) => {
                return Signal::Failure(RuntimeError::new(
                    "T-STD-00001",
                    "DecodeError",
                    format!("Failed to decode base64 string: {:?}", err),
                ));
            }
        },
        _ => return arg_err(format!("Unsupported decoding: {}", encoding)),
    };

    Signal::Success(Value {
        kind: Some(Kind::StringValue(decoded)),
    })
}

#[taurus_macros::runtime_function(
    identifier = "std::text::is_equal",
    module = "taurus-text",
    signature = "(first: TEXT, second: TEXT): BOOLEAN",
    name(en_US = "Is Equal"),
    description(en_US = "Determines if the two given text inputs are exactly the same, returning true if equal, false otherwise."),
    display_message(en_US = "${first} Equals ${second}"),
    alias(en_US = "equal;equals;same;text;string;std;is"),
    display_icon = "tabler:abc",
    linked_data_type_identifiers = ["TEXT", "BOOLEAN"],
)]
#[parameter(
    runtime_name = "first",
    name(en_US = "First Text"),
    description(en_US = "The first text to compare.")
)]
#[parameter(
    runtime_name = "second",
    name(en_US = "Second Text"),
    description(en_US = "The second text to compare.")
)]
fn is_equal(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => lhs: String, rhs: String);
    Signal::Success(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::{number_to_f64, value_from_f64, value_from_i64};
    use tucana::shared::{ListValue, Value, value::Kind};

    // ---------- helpers: build Arguments ----------
    fn a_str(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }
    fn a_num(n: f64) -> Argument {
        Argument::Eval(value_from_f64(n))
    }
    fn a_list(vals: Vec<Value>) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::ListValue(ListValue { values: vals })),
        })
    }

    // ---------- helpers: build bare Values ----------
    fn v_str(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }
    fn v_num(n: i64) -> Value {
        value_from_i64(n)
    }

    // ---------- helpers: extract from Signal ----------
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
    fn expect_str(sig: Signal) -> String {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::StringValue(s)),
            }) => s,
            other => panic!("Expected StringValue, got {:?}", other),
        }
    }
    fn expect_list(sig: Signal) -> Vec<Value> {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::ListValue(ListValue { values })),
            }) => values,
            other => panic!("Expected ListValue, got {:?}", other),
        }
    }

    // dummy runner for handlers that accept `run: &mut crate::handler::registry::ThunkRunner<'_>`
    fn dummy_run(_: &crate::handler::argument::Thunk, _: &mut ValueStore) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    // ---------- tests ----------

    #[test]
    fn test_as_bytes_and_byte_size() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;

        // "hello" -> 5 bytes
        let bytes = expect_list(as_bytes(&[a_str("hello")], &mut ctx, &mut run));
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], v_num(104)); // 'h'

        let mut run = dummy_run;
        assert_eq!(
            expect_num(byte_size(&[a_str("hello")], &mut ctx, &mut run)),
            5.0
        );

        // unicode: "café" -> 5 bytes, 4 chars
        let mut run = dummy_run;
        assert_eq!(
            expect_num(byte_size(&[a_str("café")], &mut ctx, &mut run)),
            5.0
        );
        let mut run = dummy_run;
        assert_eq!(
            expect_num(length(&[a_str("café")], &mut ctx, &mut run)),
            4.0
        );
    }

    #[test]
    fn test_case_ops_and_trim() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(capitalize(&[a_str("hello world")], &mut ctx, &mut run)),
            "Hello World"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(uppercase(&[a_str("Hello")], &mut ctx, &mut run)),
            "HELLO"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(lowercase(&[a_str("Hello")], &mut ctx, &mut run)),
            "hello"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(swapcase(&[a_str("HeLLo123")], &mut ctx, &mut run)),
            "hEllO123"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(trim(&[a_str("  hi  ")], &mut ctx, &mut run)),
            "hi"
        );
    }

    #[test]
    fn test_chars_and_at() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let chars_list = expect_list(chars(&[a_str("abc")], &mut ctx, &mut run));
        assert_eq!(chars_list, vec![v_str("a"), v_str("b"), v_str("c")]);

        let mut run = dummy_run;
        assert_eq!(
            expect_str(at(&[a_str("hello"), a_num(1.0)], &mut ctx, &mut run)),
            "e"
        );

        // out-of-bounds
        let mut run = dummy_run;
        match at(&[a_str("hi"), a_num(5.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
        // negative
        let mut run = dummy_run;
        match at(&[a_str("hi"), a_num(-1.0)], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure, got {:?}", s),
        }
    }

    #[test]
    fn test_append_prepend_insert_length() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(append(
                &[a_str("hello"), a_str(" world")],
                &mut ctx,
                &mut run
            )),
            "hello world"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(prepend(
                &[a_str("world"), a_str("hello ")],
                &mut ctx,
                &mut run
            )),
            "hello world"
        );

        // insert uses BYTE index; for ASCII this matches char index
        let mut run = dummy_run;
        assert_eq!(
            expect_str(insert(
                &[a_str("hello"), a_num(2.0), a_str("XXX")],
                &mut ctx,
                &mut run
            )),
            "heXXXllo"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(length(&[a_str("hello")], &mut ctx, &mut run)),
            5.0
        );
    }

    #[test]
    fn test_remove_replace_variants() {
        let mut ctx = ValueStore::default();

        // remove uses CHAR indices [from, to)
        let mut run = dummy_run;
        assert_eq!(
            expect_str(remove(
                &[a_str("hello world"), a_num(2.0), a_num(7.0)],
                &mut ctx,
                &mut run
            )),
            "heorld"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace(
                &[a_str("hello world hello"), a_str("hello"), a_str("hi")],
                &mut ctx,
                &mut run
            )),
            "hi world hi"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace_first(
                &[a_str("one two one"), a_str("one"), a_str("1")],
                &mut ctx,
                &mut run
            )),
            "1 two one"
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(replace_last(
                &[a_str("one two one"), a_str("one"), a_str("1")],
                &mut ctx,
                &mut run
            )),
            "one two 1"
        );
    }

    #[test]
    fn test_hex_octal_reverse() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(hex(&[a_str("hello")], &mut ctx, &mut run)),
            "68656c6c6f"
        );

        let mut run = dummy_run;
        assert_eq!(expect_str(octal(&[a_str("A")], &mut ctx, &mut run)), "101");

        let mut run = dummy_run;
        assert_eq!(
            expect_str(reverse(&[a_str("hello")], &mut ctx, &mut run)),
            "olleh"
        );
    }

    #[test]
    fn test_index_contains_split_starts_ends() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(index_of(
                &[a_str("hello world"), a_str("world")],
                &mut ctx,
                &mut run
            )),
            6.0
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_num(index_of(
                &[a_str("hello"), a_str("xyz")],
                &mut ctx,
                &mut run
            )),
            -1.0
        );

        let mut run = dummy_run;
        assert!(expect_bool(contains(
            &[a_str("hello world"), a_str("world")],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        let split_list = expect_list(split(&[a_str("a,b,c"), a_str(",")], &mut ctx, &mut run));
        assert_eq!(split_list, vec![v_str("a"), v_str("b"), v_str("c")]);

        let mut run = dummy_run;
        assert!(expect_bool(starts_with(
            &[a_str("hello"), a_str("he")],
            &mut ctx,
            &mut run
        )));

        let mut run = dummy_run;
        assert!(expect_bool(ends_with(
            &[a_str("hello"), a_str("lo")],
            &mut ctx,
            &mut run
        )));
    }

    #[test]
    fn test_to_ascii_and_from_ascii() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        let ascii_vals = expect_list(to_ascii(&[a_str("AB")], &mut ctx, &mut run));
        assert_eq!(ascii_vals, vec![v_num(65), v_num(66)]);

        let mut run = dummy_run;
        let list_arg = a_list(vec![v_num(65), v_num(66), v_num(67)]);
        assert_eq!(
            expect_str(from_ascii(&[list_arg], &mut ctx, &mut run)),
            "ABC"
        );

        // invalid element
        let mut run = dummy_run;
        let list_arg = a_list(vec![v_num(65), v_num(128)]);
        match from_ascii(&[list_arg], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for invalid ASCII, got {:?}", s),
        }
    }

    #[test]
    fn test_encode_decode_base64_and_is_equal() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_str(encode(
                &[a_str("hello"), a_str("BASE64")],
                &mut ctx,
                &mut run
            )),
            "aGVsbG8="
        );

        let mut run = dummy_run;
        assert_eq!(
            expect_str(decode(
                &[a_str("aGVsbG8="), a_str("base64")],
                &mut ctx,
                &mut run
            )),
            "hello"
        );

        // unsupported codec
        let mut run = dummy_run;
        match encode(&[a_str("data"), a_str("gug")], &mut ctx, &mut run) {
            Signal::Failure(_) => {}
            s => panic!("Expected Failure for unsupported encoding, got {:?}", s),
        }

        let mut run = dummy_run;
        assert!(expect_bool(is_equal(
            &[a_str("x"), a_str("x")],
            &mut ctx,
            &mut run
        )));
        let mut run = dummy_run;
        assert!(!expect_bool(is_equal(
            &[a_str("x"), a_str("y")],
            &mut ctx,
            &mut run
        )));
    }
}
