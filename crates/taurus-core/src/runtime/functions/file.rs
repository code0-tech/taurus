//! File standard-library handlers.
//!
//! `FILE<M>` is a storage reference: a struct
//! `{ contentType: M; fileName: string; size: NUMBER; checksum: string;
//! valueType: 'reference'; value: string }` where `value` is an opaque storage
//! handle (not content, not a URL). Metadata is trusted; bytes are fetched lazily
//! (not yet implemented, see taurus#363).

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::signal::Signal;
use crate::value::{number_to_i64_lossy, value_from_i64};
use tucana::shared::{Struct, value::Kind};

// No `definitions/taurus-file/*.json` counterpart exists in this repo -- see
// the note in `date.rs`. The metadata below is transcribed from Aquila's
// live definitions instead.
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
    alias(en_US = "file;document;blob;attachment;binary"),
    generic_keys = ["M extends TEXT"],
    type_string = "{ contentType: M; fileName: string; size: NUMBER; checksum: string; valueType: 'reference'; value: string }",
    linked_data_type_identifiers = ["TEXT"],
}

#[taurus_macros::runtime_function(
    identifier = "std::file::size",
    module = "taurus-file",
    signature = "<M extends TEXT>(file: FILE<M>): NUMBER",
    name(en_US = "File Size"),
    description(en_US = "Returns the size of the given file in bytes."),
    display_message(en_US = "Size of ${file}"),
    alias(en_US = "size;file;bytes;length;document;blob;std"),
    display_icon = "tabler:file",
    linked_data_type_identifiers = ["FILE", "NUMBER", "TEXT"],
)]
#[parameter(
    runtime_name = "file",
    name(en_US = "File"),
    description(en_US = "The file whose size in bytes is to be returned.")
)]
fn size(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => file: Struct);

    // `throwsError: false` -- a missing/malformed `size` field reports 0.
    let byte_len = match file.fields.get("size").and_then(|v| v.kind.as_ref()) {
        Some(Kind::NumberValue(n)) => number_to_i64_lossy(n).unwrap_or(0),
        _ => 0,
    };

    Signal::Success(value_from_i64(byte_len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tucana::shared::Value;

    struct DummyRun;
    impl crate::handler::registry::AsyncThunkRunner for DummyRun {
        fn run_thunk<'a>(
            &'a mut self,
            _: &'a crate::handler::argument::Thunk,
            _: &'a mut ValueStore,
        ) -> crate::handler::registry::HandlerFuture<'a> {
            Box::pin(async {
                Signal::Success(Value {
                    kind: Some(Kind::NullValue(0)),
                })
            })
        }
    }
    #[allow(non_upper_case_globals)]
    const dummy_run: DummyRun = DummyRun;

    fn expect_num(sig: Signal) -> i64 {
        match sig {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_i64_lossy(&n).unwrap_or_default(),
            other => panic!("Expected NumberValue, got {:?}", other),
        }
    }

    fn file_value(content_type: &str, size: Option<i64>) -> Argument {
        let mut fields = HashMap::new();
        let mut put = |k: &str, v: Value| {
            fields.insert(k.to_string(), v);
        };
        let string = |v: &str| Value {
            kind: Some(Kind::StringValue(v.to_string())),
        };
        put("contentType", string(content_type));
        put("fileName", string("a.txt"));
        put("checksum", string("sha256:abc"));
        put("valueType", string("reference"));
        put("value", string("handle-123"));
        if let Some(size) = size {
            put("size", value_from_i64(size));
        }
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(Struct { fields })),
        })
    }

    #[test]
    fn test_size_reads_size_field() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        assert_eq!(
            expect_num(size(
                &[file_value("text/plain", Some(11))],
                &mut ctx,
                &mut run
            )),
            11
        );
    }

    #[test]
    fn test_size_returns_zero_when_size_missing() {
        let mut ctx = ValueStore::default();
        let mut run = dummy_run;
        assert_eq!(
            expect_num(size(&[file_value("text/plain", None)], &mut ctx, &mut run)),
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
