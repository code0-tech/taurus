//! HTTP and REST-oriented helper handlers.
//!
//! These functions build/validate plain struct payloads that the runtime treats as regular values.

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use crate::value::number_to_string;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Read;
use tucana::shared::helper::value::{from_json_value, to_json_value, ToValue};
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("http::request::create", create_request, 4),
    FunctionRegistration::eager("http::request::send", send_request, 1),
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

fn send_request(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => http_request: Struct);

    let method = match expect_struct_string_field(&http_request, "http_method") {
        Ok(value) => value,
        Err(signal) => return signal,
    };
    let url = match expect_struct_string_field(&http_request, "url") {
        Ok(value) => value,
        Err(signal) => return signal,
    };
    let headers_struct = match expect_struct_struct_field(&http_request, "headers") {
        Ok(value) => value,
        Err(signal) => return signal,
    };
    let payload = match http_request.fields.get("payload") {
        Some(value) => value.clone(),
        None => {
            return fail(
                "InvalidArgumentRuntimeError",
                "Missing 'payload' field in http_request",
            );
        }
    };

    let mut headers = match encode_headers(&headers_struct) {
        Ok(headers) => headers,
        Err(message) => return fail("InvalidArgumentRuntimeError", message),
    };

    let request_content_type = content_type_header_value(&headers);
    let (request_body, default_content_type) =
        match encode_request_payload(&payload, request_content_type.as_deref()) {
            Ok(result) => result,
            Err(message) => return fail("InvalidArgumentRuntimeError", message),
        };

    if let Some(default_content_type) = default_content_type
        && request_content_type.is_none()
    {
        headers.insert("content-type".to_string(), default_content_type.to_string());
    }

    let mut request = ureq::request(&method, &url);
    for (name, value) in &headers {
        request = request.set(name, value);
    }

    let response_result = match request_body {
        Some(bytes) => request.send_bytes(bytes.as_slice()),
        None => request.call(),
    };

    let response = match response_result {
        Ok(response) => response,
        Err(ureq::Error::Status(_, response)) => response,
        Err(ureq::Error::Transport(err)) => {
            return fail(
                "HttpRequestRuntimeError",
                format!("HTTP transport error while sending request: {}", err),
            );
        }
    };

    let status_code = response.status() as i64;
    let response_headers = decode_headers(&response);
    let response_payload = match decode_response_payload(response) {
        Ok(result) => result,
        Err(message) => return fail("HttpRequestRuntimeError", message),
    };

    let mut fields = HashMap::new();
    fields.insert("http_status_code".to_string(), status_code.to_value());
    fields.insert("headers".to_string(), response_headers.to_value());
    fields.insert("payload".to_string(), response_payload);

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

fn expect_struct_string_field(struct_val: &Struct, field: &str) -> Result<String, Signal> {
    let Some(value) = struct_val.fields.get(field) else {
        return Err(fail(
            "InvalidArgumentRuntimeError",
            format!("Missing '{}' field in http_request", field),
        ));
    };

    match &value.kind {
        Some(Kind::StringValue(str_val)) => Ok(str_val.clone()),
        _ => Err(fail(
            "InvalidArgumentRuntimeError",
            format!("Expected '{}' to be StringValue", field),
        )),
    }
}

fn expect_struct_struct_field(struct_val: &Struct, field: &str) -> Result<Struct, Signal> {
    let Some(value) = struct_val.fields.get(field) else {
        return Err(fail(
            "InvalidArgumentRuntimeError",
            format!("Missing '{}' field in http_request", field),
        ));
    };

    match &value.kind {
        Some(Kind::StructValue(struct_val)) => Ok(struct_val.clone()),
        _ => Err(fail(
            "InvalidArgumentRuntimeError",
            format!("Expected '{}' to be StructValue", field),
        )),
    }
}

fn encode_headers(headers: &Struct) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::with_capacity(headers.fields.len());
    for (name, value) in &headers.fields {
        if name.trim().is_empty() {
            return Err("Header name cannot be empty".to_string());
        }
        out.insert(name.clone(), value_to_string(value)?);
    }
    Ok(out)
}

fn value_to_string(value: &Value) -> Result<String, String> {
    match &value.kind {
        Some(Kind::StringValue(str_val)) => Ok(str_val.clone()),
        Some(Kind::NumberValue(number)) => Ok(number_to_string(number)),
        Some(Kind::BoolValue(bool_val)) => Ok(bool_val.to_string()),
        Some(Kind::NullValue(_)) | None => Err("Null is not a valid header value".to_string()),
        Some(Kind::ListValue(_)) | Some(Kind::StructValue(_)) => {
            serde_json::to_string(&to_json_value(value.clone()))
                .map_err(|err| format!("Unable to serialize header value: {}", err))
        }
    }
}

fn content_type_header_value(headers: &HashMap<String, String>) -> Option<String> {
    headers.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case("content-type") {
            Some(value.clone())
        } else {
            None
        }
    })
}

fn normalize_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn content_type_is_text_plain(content_type: &str) -> bool {
    content_type == "text/plain"
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestBodyEncoding {
    Json,
    TextPlain,
}

fn resolve_request_body_encoding(
    payload: &Value,
    request_content_type: Option<&str>,
) -> Result<Option<RequestBodyEncoding>, String> {
    if let Some(content_type) = request_content_type {
        let normalized = normalize_content_type(content_type);
        if content_type_is_json(&normalized) {
            return Ok(Some(RequestBodyEncoding::Json));
        }
        if content_type_is_text_plain(&normalized) {
            return Ok(Some(RequestBodyEncoding::TextPlain));
        }
        return Err(format!(
            "Unsupported content-type '{}' for http::request::send. Supported types: application/json, text/plain",
            content_type
        ));
    }

    match payload.kind.as_ref() {
        Some(Kind::NullValue(_)) | None => Ok(None),
        Some(Kind::StringValue(_)) => Ok(Some(RequestBodyEncoding::TextPlain)),
        _ => Ok(Some(RequestBodyEncoding::Json)),
    }
}

fn encode_request_payload(
    payload: &Value,
    request_content_type: Option<&str>,
) -> Result<(Option<Vec<u8>>, Option<&'static str>), String> {
    let Some(encoding) = resolve_request_body_encoding(payload, request_content_type)? else {
        return Ok((None, None));
    };

    match encoding {
        RequestBodyEncoding::Json => {
            let json = to_json_value(payload.clone());
            let body = serde_json::to_vec(&json)
                .map_err(|err| format!("Unable to serialize request payload: {}", err))?;
            Ok((Some(body), Some("application/json")))
        }
        RequestBodyEncoding::TextPlain => match payload.kind.as_ref() {
            Some(Kind::NullValue(_)) | None => Ok((None, Some("text/plain"))),
            Some(Kind::StringValue(body)) => {
                Ok((Some(body.as_bytes().to_vec()), Some("text/plain")))
            }
            _ => Err("Payload must be StringValue when content-type is text/plain".to_string()),
        },
    }
}

fn decode_headers(response: &ureq::Response) -> Struct {
    let mut fields = HashMap::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            fields.insert(name, value.to_string().to_value());
        }
    }
    Struct { fields }
}

fn decode_response_payload(response: ureq::Response) -> Result<Value, String> {
    let content_type = response
        .header("content-type")
        .map(|value| value.to_ascii_lowercase());

    let mut bytes = Vec::new();
    let mut reader = response.into_reader();
    reader
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Unable to read HTTP response payload: {}", err))?;

    if bytes.is_empty() {
        return Ok(Value {
            kind: Some(Kind::NullValue(0)),
        });
    }

    if let Ok(text) = String::from_utf8(bytes.clone()) {
        if content_type
            .as_deref()
            .map(content_type_is_json)
            .unwrap_or(false)
        {
            if let Ok(json) = serde_json::from_str::<JsonValue>(&text) {
                return Ok(from_json_value(json));
            }
        }

        return Ok(text.to_value());
    }

    let values: Vec<i64> = bytes.iter().map(|byte| *byte as i64).collect();
    Ok(values.to_value())
}

fn content_type_is_json(content_type: &str) -> bool {
    content_type.contains("/json") || content_type.contains("+json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::argument::Argument;
    use crate::runtime::execution::value_store::ValueStore;
    use crate::value::number_to_i64_lossy;
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    fn string_value(value: &str) -> Value {
        value.to_string().to_value()
    }

    #[test]
    fn encode_request_payload_serializes_non_string_values_to_json() {
        let payload = Value {
            kind: Some(Kind::StructValue(Struct {
                fields: HashMap::from([(
                    "ok".to_string(),
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                )]),
            })),
        };

        let (body, content_type) = match encode_request_payload(&payload, None) {
            Ok(result) => result,
            Err(err) => panic!("unexpected error: {}", err),
        };

        assert_eq!(content_type, Some("application/json"));
        let body = body.unwrap_or_default();
        let text = match String::from_utf8(body) {
            Ok(text) => text,
            Err(err) => panic!("payload was not valid utf8: {}", err),
        };
        let decoded = serde_json::from_str::<JsonValue>(&text).unwrap_or(JsonValue::Null);
        let JsonValue::Object(map) = decoded else {
            panic!("expected encoded payload to be json object");
        };
        let Some(JsonValue::Bool(ok)) = map.get("ok") else {
            panic!("missing ok field in json payload");
        };
        assert!(*ok);

        let (empty_body, empty_content_type) = encode_request_payload(
            &Value {
                kind: Some(Kind::NullValue(0)),
            },
            None,
        )
        .unwrap_or((Some(vec![1]), Some("application/json")));
        assert_eq!(empty_content_type, None);
        assert!(empty_body.is_none());
    }

    #[test]
    fn encode_request_payload_uses_text_plain_header_and_rejects_unsupported_content_type() {
        let (text_body, text_content_type) =
            encode_request_payload(&string_value("hello"), Some("text/plain; charset=utf-8"))
                .unwrap_or((None, None));

        assert_eq!(text_content_type, Some("text/plain"));
        let body = text_body.unwrap_or_default();
        assert_eq!(body, b"hello");

        let err = encode_request_payload(&string_value("hello"), Some("application/xml"));
        let Err(err) = err else {
            panic!("expected unsupported content-type error");
        };
        assert!(err.contains("Supported types: application/json, text/plain"));

        let err = encode_request_payload(
            &Value {
                kind: Some(Kind::NullValue(0)),
            },
            Some("application/octet-stream"),
        );
        let Err(err) = err else {
            panic!("expected unsupported content-type error for null payload");
        };
        assert!(err.contains("Supported types: application/json, text/plain"));

        let err = encode_request_payload(
            &Value {
                kind: Some(Kind::StructValue(Struct {
                    fields: HashMap::from([("a".to_string(), 1i64.to_value())]),
                })),
            },
            Some("text/plain"),
        );
        let Err(err) = err else {
            panic!("expected text/plain payload validation error");
        };
        assert!(err.contains("Payload must be StringValue"));
    }

    #[test]
    fn encode_headers_rejects_null_values() {
        let headers = Struct {
            fields: HashMap::from([(
                "x-null".to_string(),
                Value {
                    kind: Some(Kind::NullValue(0)),
                },
            )]),
        };

        let result = encode_headers(&headers);
        let Err(err) = result else {
            panic!("expected error for null header value");
        };
        assert!(err.contains("Null is not a valid header value"));
    }

    #[test]
    fn send_request_tcp_listener_roundtrip_validates_request_and_response_mapping() {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) => panic!("failed to bind test listener: {}", err),
        };
        let addr = match listener.local_addr() {
            Ok(addr) => addr,
            Err(err) => panic!("failed to fetch local address: {}", err),
        };

        let server = thread::spawn(move || {
            let (mut stream, _) = match listener.accept() {
                Ok(pair) => pair,
                Err(err) => panic!("failed to accept inbound socket: {}", err),
            };

            if let Err(err) = stream.set_read_timeout(Some(Duration::from_secs(3))) {
                panic!("failed to configure socket timeout: {}", err);
            }

            let mut request_bytes = Vec::new();
            let mut buf = [0_u8; 1024];
            let mut headers_end = None;
            let mut content_length = 0_usize;

            loop {
                let n = match stream.read(&mut buf) {
                    Ok(n) => n,
                    Err(err) => panic!("failed while reading request bytes: {}", err),
                };
                if n == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buf[..n]);

                if headers_end.is_none() {
                    headers_end = request_bytes.windows(4).position(|w| w == b"\r\n\r\n");
                    if let Some(idx) = headers_end {
                        let header_text = match String::from_utf8(request_bytes[..idx].to_vec()) {
                            Ok(text) => text,
                            Err(err) => panic!("request headers not utf8: {}", err),
                        };
                        for line in header_text.lines().skip(1) {
                            if let Some((name, value)) = line.split_once(':')
                                && name.eq_ignore_ascii_case("content-length")
                            {
                                content_length = value.trim().parse::<usize>().unwrap_or(0);
                            }
                        }
                    }
                }

                if let Some(idx) = headers_end {
                    let body_start = idx + 4;
                    if request_bytes.len() >= body_start + content_length {
                        break;
                    }
                }
            }

            let Some(headers_end) = headers_end else {
                panic!("did not receive full header block");
            };
            let body_start = headers_end + 4;
            let header_text = match String::from_utf8(request_bytes[..headers_end].to_vec()) {
                Ok(text) => text,
                Err(err) => panic!("request headers not utf8: {}", err),
            };
            let body_bytes = &request_bytes[body_start..];
            let body_text = match String::from_utf8(body_bytes.to_vec()) {
                Ok(text) => text,
                Err(err) => panic!("request body not utf8: {}", err),
            };

            let mut header_map = HashMap::<String, String>::new();
            let mut lines = header_text.lines();
            let start_line = lines.next().unwrap_or_default().to_string();
            for line in lines {
                if let Some((name, value)) = line.split_once(':') {
                    header_map.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
                }
            }

            assert_eq!(start_line, "POST /echo?x=1 HTTP/1.1");
            assert_eq!(
                header_map.get("x-bool").map(String::as_str),
                Some("true"),
                "expected bool header conversion to string"
            );
            assert_eq!(
                header_map.get("content-type").map(String::as_str),
                Some("application/json"),
                "expected automatic JSON content type for structured payload"
            );

            let json = serde_json::from_str::<JsonValue>(&body_text).unwrap_or(JsonValue::Null);
            let JsonValue::Object(map) = json else {
                panic!("request body should be json object");
            };
            assert_eq!(
                map.get("msg"),
                Some(&JsonValue::String("hello".to_string()))
            );
            assert_eq!(
                map.get("count"),
                Some(&JsonValue::Number(serde_json::Number::from(2)))
            );

            let response_body = r#"{"ok":true,"echo":"done"}"#;
            let response = format!(
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nX-Reply: ok\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            if let Err(err) = stream.write_all(response.as_bytes()) {
                panic!("failed sending response: {}", err);
            }
        });

        let request_payload = Value {
            kind: Some(Kind::StructValue(Struct {
                fields: HashMap::from([
                    ("msg".to_string(), string_value("hello")),
                    ("count".to_string(), 2i64.to_value()),
                ]),
            })),
        };
        let request_headers = Struct {
            fields: HashMap::from([("x-bool".to_string(), true.to_value())]),
        };
        let request = Struct {
            fields: HashMap::from([
                ("http_method".to_string(), string_value("POST")),
                (
                    "url".to_string(),
                    string_value(&format!("http://{}/echo?x=1", addr)),
                ),
                (
                    "headers".to_string(),
                    Value {
                        kind: Some(Kind::StructValue(request_headers)),
                    },
                ),
                ("payload".to_string(), request_payload),
            ]),
        };

        let args = vec![Argument::Eval(Value {
            kind: Some(Kind::StructValue(request)),
        })];
        let mut ctx = ValueStore::default();
        let mut run = |_: i64, _: &mut ValueStore| Signal::Stop;

        let signal = send_request(&args, &mut ctx, &mut run);

        let response = match signal {
            Signal::Success(Value {
                kind: Some(Kind::StructValue(response)),
            }) => response,
            other => panic!("expected success struct response, got: {:?}", other),
        };

        let status = match response.fields.get("http_status_code") {
            Some(Value {
                kind: Some(Kind::NumberValue(number)),
            }) => number_to_i64_lossy(number).unwrap_or_default(),
            _ => panic!("expected numeric status code"),
        };
        assert_eq!(status, 201);

        match response.fields.get("headers") {
            Some(Value {
                kind: Some(Kind::StructValue(headers)),
            }) => {
                let reply = headers
                    .fields
                    .get("x-reply")
                    .and_then(|value| value.kind.as_ref())
                    .and_then(|kind| match kind {
                        Kind::StringValue(value) => Some(value.as_str()),
                        _ => None,
                    });
                assert_eq!(reply, Some("ok"));
            }
            _ => panic!("expected response headers struct"),
        }

        match response.fields.get("payload") {
            Some(Value {
                kind: Some(Kind::StructValue(payload)),
            }) => {
                let ok = payload
                    .fields
                    .get("ok")
                    .and_then(|value| value.kind.as_ref())
                    .and_then(|kind| match kind {
                        Kind::BoolValue(value) => Some(*value),
                        _ => None,
                    });
                assert_eq!(ok, Some(true));

                let echo = payload
                    .fields
                    .get("echo")
                    .and_then(|value| value.kind.as_ref())
                    .and_then(|kind| match kind {
                        Kind::StringValue(value) => Some(value.as_str()),
                        _ => None,
                    });
                assert_eq!(echo, Some("done"));
            }
            _ => panic!("expected JSON response payload struct"),
        }

        if let Err(err) = server.join() {
            panic!("server thread join failed: {:?}", err);
        }
    }
}
