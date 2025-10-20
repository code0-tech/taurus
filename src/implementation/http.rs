use crate::context::Context;
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use crate::registry::HandlerFn;
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};

pub fn collect_http_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("http::request::create", create_request),
        ("http::response::create", create_response),
        ("http::control::respond", respond),
    ]
}

fn respond(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StructValue(struct_val)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected exactly one response struct, got {:?}", values),
        ));
    };

    let fields = &struct_val.fields;

    let Some(headers_val) = fields.get("headers") else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Missing 'headers' field".to_string(),
        ));
    };

    let Some(status_code_val) = fields.get("status_code") else {
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
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            "Expected 'headers' to be StructValue".to_string(),
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

fn create_request(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StringValue(http_method)),
        },
        Value {
            kind: Some(Kind::StructValue(headers)),
        },
        Value {
            kind: Some(Kind::StringValue(http_url)),
        },
        payload,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected [method, headers, url, payload] but received {:?}",
                values
            ),
        ));
    };

    let mut fields = std::collections::HashMap::new();

    fields.insert(
        "method".to_string(),
        Value {
            kind: Some(Kind::StringValue(http_method.clone())),
        },
    );

    fields.insert(
        "url".to_string(),
        Value {
            kind: Some(Kind::StringValue(http_url.clone())),
        },
    );

    fields.insert(
        "headers".to_string(),
        Value {
            kind: Some(Kind::StructValue(headers.clone())),
        },
    );
    fields.insert("body".to_string(), payload.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

fn create_response(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::NumberValue(http_status_code)),
        },
        Value {
            kind: Some(Kind::StructValue(headers)),
        },
        payload,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected [http_status_code, headers, payload] but received {:?}",
                values
            ),
        ));
    };

    let mut fields = std::collections::HashMap::new();

    fields.insert(
        "status_code".to_string(),
        Value {
            kind: Some(Kind::NumberValue(http_status_code.clone())),
        },
    );

    fields.insert(
        "headers".to_string(),
        Value {
            kind: Some(Kind::StructValue(headers.clone())),
        },
    );
    fields.insert("payload".to_string(), payload.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

