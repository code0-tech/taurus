use crate::context::argument::Argument;
use crate::context::context::Context;
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::runtime::error::RuntimeError;
use tucana::shared::helper::value::ToValue;
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};

pub fn collect_http_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("http::request::create", HandlerFn::eager(create_request, 1)),
        (
            "http::response::create",
            HandlerFn::eager(create_response, 4),
        ),
        ("rest::control::respond", HandlerFn::eager(respond, 1)),
    ]
}

fn respond(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => struct_val: Struct);

    let fields = &struct_val.fields;

    let Some(headers_val) = fields.get("headers") else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Missing 'headers' field".to_string(),
        ));
    };

    let Some(status_code_val) = fields.get("http_status_code") else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Missing 'status_code' field".to_string(),
        ));
    };

    let Some(payload_val) = fields.get("payload") else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Missing 'payload' field".to_string(),
        ));
    };

    let Some(Kind::StructValue(_headers_struct)) = &headers_val.kind else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected 'headers' to be StructValue",
        ));
    };

    let Some(Kind::NumberValue(_status_code_str)) = &status_code_val.kind else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Expected 'status_code' to be NumberValue".to_string(),
        ));
    };

    let Some(_payload_kind) = &payload_val.kind else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Expected 'payload' to have a value".to_string(),
        ));
    };

    Signal::Respond(Value {
        kind: Some(Kind::StructValue(struct_val.clone())),
    })
}

fn create_request(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
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
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
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
