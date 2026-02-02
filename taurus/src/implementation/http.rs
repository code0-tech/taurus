use crate::context::argument::Argument;
use crate::context::context::Context;
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use tucana::shared::value::Kind;
use tucana::shared::{ListValue, Struct, Value};

pub fn collect_http_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("http::request::create", HandlerFn::eager(create_request, 1)),
        (
            "http::response::create",
            HandlerFn::eager(create_response, 4),
        ),
        ("rest::control::respond", HandlerFn::eager(respond, 3)),
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

    let Some(Kind::ListValue(_headers_struct)) = &headers_val.kind else {
        return Signal::Failure(RuntimeError::simple_str(
            "InvalidArgumentRuntimeError",
            "Expected 'headers' to be ListValue",
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
    args!(args => http_method: String, headers: ListValue, http_url: String, payload: Value);
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
            kind: Some(Kind::ListValue(headers.clone())),
        },
    );
    fields.insert("body".to_string(), payload.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

fn create_response(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => http_status_code: String, headers: ListValue, payload: Value);
    let mut fields = std::collections::HashMap::new();

    let code = match http_status_code.as_str().parse::<f64>() {
        Ok(c) => c,
        Err(_) => {
            return Signal::Failure(RuntimeError::simple_str(
                "InvalidArgumentExeption",
                "Expected http_status_code to be parsed to float",
            ));
        }
    };
    fields.insert(
        "status_code".to_string(),
        Value {
            kind: Some(Kind::NumberValue(code)),
        },
    );

    fields.insert(
        "headers".to_string(),
        Value {
            kind: Some(Kind::ListValue(headers.clone())),
        },
    );
    fields.insert("payload".to_string(), payload.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}
