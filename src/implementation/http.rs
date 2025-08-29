use tucana::shared::{Struct, Value};
use tucana::shared::value::Kind;
use crate::context::Context;
use crate::error::RuntimeError;
use crate::registry::HandlerFn;

pub fn collect_http_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("http::request::create", create_request),
        ("http::response::create", create_response),
    ]
}

fn create_request(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(http_method)),
    },
    Value {
        kind: Some(Kind::StructValue(headers)),
    },
    Value {
        kind: Some(Kind::StringValue(http_url)),
    },
    payload
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected [method, headers, url, payload] but received {:?}", values),
        ));
    };

    let mut fields = std::collections::HashMap::new();

    fields.insert("method".to_string(), Value {
        kind: Some(Kind::StringValue(http_method.clone())),
    });

    fields.insert("url".to_string(), Value {
        kind: Some(Kind::StringValue(http_url.clone())),
    });

    fields.insert("headers".to_string(),Value {
        kind: Some(Kind::StructValue(headers.clone())),
    });
    fields.insert("body".to_string(), payload.clone());

    Ok(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

fn create_response(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(http_status_code)),
    },
    Value {
        kind: Some(Kind::StructValue(headers)),
    },
    payload
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected [http_status_code, headers, payload] but received {:?}", values),
        ));
    };

    let mut fields = std::collections::HashMap::new();

    fields.insert("method".to_string(), Value {
        kind: Some(Kind::NumberValue(http_status_code.clone())),
    });

    fields.insert("headers".to_string(),Value {
        kind: Some(Kind::StructValue(headers.clone())),
    });
    fields.insert("body".to_string(), payload.clone());

    Ok(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}