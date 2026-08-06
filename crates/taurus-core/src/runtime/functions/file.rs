//! File standard-library handlers.
//!
//! `FILE<M>` is a struct `{ contentType: M; valueType: 'base64'; value: string }`.
//! Only `value` (the base64 payload) matters for these handlers; `contentType`/
//! `valueType` are carried through untouched by callers.

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::signal::Signal;
use crate::value::value_from_i64;
use base64::Engine;
use tucana::shared::{Struct, value::Kind};

// No `definitions/taurus-file/*.json` counterpart exists to port from -- see
// the note in `date.rs`, which is in the same situation.
taurus_macros::module! {
    identifier = "taurus-file",
    name(en_US = "File"),
    description(en_US = "Work with Files."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:file",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "FILE",
    module = "taurus-file",
    name(en_US = "File"),
    display_message(en_US = "File"),
    alias(en_US = "file;attachment;blob"),
    generic_keys = ["M"],
    type_string = "{ contentType: M, valueType: 'base64', value: TEXT }",
    linked_data_type_identifiers = ["TEXT"],
}

#[taurus_macros::runtime_function(
    identifier = "std::file::size",
    module = "taurus-file",
    signature = "<M>(file: FILE<M>): NUMBER",
    name(en_US = "File Size"),
    description(en_US = "Returns the decoded byte size of a file's base64 payload."),
    display_message(en_US = "Size of ${file}"),
    alias(en_US = "size;file;bytes;length;std"),
    display_icon = "tabler:file",
    linked_data_type_identifiers = ["FILE", "NUMBER"],
)]
#[parameter(
    runtime_name = "file",
    name(en_US = "File"),
    description(en_US = "The file whose decoded byte size to compute.")
)]
fn size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => file: Struct);

    // `throwsError: false` — a missing/malformed `value` field reports size 0
    // rather than failing.
    let byte_len = match file.fields.get("value").and_then(|v| v.kind.as_ref()) {
        Some(Kind::StringValue(base64_value)) => base64::prelude::BASE64_STANDARD
            .decode(base64_value)
            .map(|bytes| bytes.len())
            .unwrap_or(0),
        _ => 0,
    };

    Signal::Success(value_from_i64(byte_len as i64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::number_to_i64_lossy;
    use base64::Engine;
    use std::collections::HashMap;
    use tucana::shared::Value;

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

    fn file_value(content_type: &str, value: &str) -> Argument {
        let mut fields = HashMap::new();
        fields.insert(
            "contentType".to_string(),
            Value {
                kind: Some(Kind::StringValue(content_type.to_string())),
            },
        );
        fields.insert(
            "valueType".to_string(),
            Value {
                kind: Some(Kind::StringValue("base64".to_string())),
            },
        );
        fields.insert(
            "value".to_string(),
            Value {
                kind: Some(Kind::StringValue(value.to_string())),
            },
        );
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(Struct { fields })),
        })
    }

    #[test]
    fn test_size_decodes_base64_payload() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        let encoded = base64::prelude::BASE64_STANDARD.encode("hello world");

        assert_eq!(
            expect_num(size(
                &[file_value("text/plain", &encoded)],
                &mut ctx,
                &mut run
            )),
            11
        );
    }

    #[test]
    fn test_size_returns_zero_for_malformed_input() {
        let mut ctx = ValueStore::default();

        let mut run = dummy_run;
        assert_eq!(
            expect_num(size(
                &[file_value("text/plain", "not-valid-base64!!")],
                &mut ctx,
                &mut run
            )),
            0
        );

        let mut run = dummy_run;
        let empty_struct = Argument::Eval(Value {
            kind: Some(Kind::StructValue(Struct {
                fields: HashMap::new(),
            })),
        });
        assert_eq!(expect_num(size(&[empty_struct], &mut ctx, &mut run)), 0);
    }
}
