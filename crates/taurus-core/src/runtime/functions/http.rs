//! HTTP and REST-oriented helper handlers.
//!
//! These functions build/validate plain struct payloads that the runtime treats as regular values.

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use tucana::shared::helper::value::ToValue;
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("http::request::create", create_request, 4),
    FunctionRegistration::eager("http::response::create", create_response, 3),
    FunctionRegistration::eager("rest::control::respond", respond, 1),
];

fn fail(category: &str, message: impl Into<String>) -> Signal {
    Signal::Failure(RuntimeError::new("T-STD-00001", category, message))
}

fn respond(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => struct_val: Struct);

    let fields = &struct_val.fields;

    let Some(headers_val) = fields.get("headers") else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Missing 'headers' field".to_string(),
        ));
    };

    let Some(status_code_val) = fields.get("http_status_code") else {
        return fail(
            "InvalidArgumentRuntimeError",
            "Missing 'http_status_code' field",
        );
    };

    let Some(payload_val) = fields.get("payload") else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Missing 'payload' field".to_string(),
        ));
    };

    let Some(Kind::StructValue(_headers_struct)) = &headers_val.kind else {
        return fail(
            "InvalidArgumentRuntimeError",
            "Expected 'headers' to be StructValue",
        );
    };

    let Some(Kind::NumberValue(_status_code_str)) = &status_code_val.kind else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected 'status_code' to be NumberValue".to_string(),
        ));
    };

    let Some(_payload_kind) = &payload_val.kind else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            "Expected 'payload' to have a value".to_string(),
        ));
    };

    // `Respond` is a control signal; the executor can still continue with `next` if present.
    Signal::Respond(Value {
        kind: Some(Kind::StructValue(struct_val.clone())),
    })
}

fn create_request(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => http_method: String, headers: Struct, http_url: String, payload: Value);
    let mut fields = std::collections::HashMap::new();

    fields.insert(String::from("http_method"), http_method.to_value());
    fields.insert(String::from("url"), http_url.to_value());
    fields.insert(String::from("payload"), payload.clone());
    fields.insert(
        String::from("headers"),
        Value {
            kind: Some(Kind::StructValue(headers.clone())),
        },
    );

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

fn create_response(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => http_status_code: i64, headers: Struct, payload: Value);
    let mut fields = std::collections::HashMap::new();

    fields.insert(
        String::from("http_status_code"),
        http_status_code.to_value(),
    );
    fields.insert(String::from("payload"), payload.clone());

    fields.insert(
        String::from("headers"),
        Value {
            kind: Some(Kind::StructValue(headers.clone())),
        },
    );

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}
