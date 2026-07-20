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
use base64::Engine;
use lupus::{DecodeContext, EncodeContext, Engine as ConversionEngine, Format};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Read;
use tucana::shared::helper::value::{ToValue, from_json_value, to_json_value};
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};
use ureq::http;
use ureq::{Body, RequestExt};

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("http::request::send", send_request, 8),
    FunctionRegistration::eager("rest::control::respond", respond, 4),
];

fn fail(category: &str, message: impl Into<String>) -> Signal {
    Signal::Failure(RuntimeError::new("T-STD-00001", category, message))
}

fn respond(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => http_status_code: i64, http_schema: String, payload: Value, headers: Value);

    let http_headers = match headers_from_value(&headers) {
        Ok(headers) => headers,
        Err(signal) => return signal,
    };

    let mut fields = HashMap::new();
    fields.insert("http_status_code".to_string(), http_status_code.to_value());
    fields.insert(
        "headers".to_string(),
        Value {
            kind: Some(Kind::StructValue(http_headers)),
        },
    );

    fields.insert("payload".to_string(), payload);
    fields.insert("http_schema".to_string(), http_schema.to_value());

    // `Respond` is a control signal; the executor can still continue with `next` if present.
    Signal::Respond(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum HttpAuthType {
    None,
    Bearer,
    Basic,
    XApiKey,
    Custom(String),
}

impl HttpAuthType {
    fn from_value(input: &Value) -> Result<HttpAuthType, String> {
        match input.kind.as_ref() {
            Some(Kind::NullValue(_)) | None => Ok(HttpAuthType::None),
            Some(Kind::StringValue(value)) => {
                if value.eq_ignore_ascii_case("bearer") {
                    Ok(HttpAuthType::Bearer)
                } else if value.eq_ignore_ascii_case("basic") {
                    Ok(HttpAuthType::Basic)
                } else if value.eq_ignore_ascii_case("x-api-key") {
                    Ok(HttpAuthType::XApiKey)
                } else {
                    match value.as_str() {
                        "undefined" | "" => Ok(HttpAuthType::None),
                        custom => Ok(HttpAuthType::Custom(custom.to_string())),
                    }
                }
            }
            _ => Err("Auth Type must be a string or undefined".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HttpAuthPlace {
    Header,
    Url,
}

impl HttpAuthPlace {
    fn from_value(input: &Value) -> Result<Option<HttpAuthPlace>, String> {
        match input.kind.as_ref() {
            Some(Kind::NullValue(_)) | None => Ok(None),
            Some(Kind::StringValue(value)) => {
                if value.eq_ignore_ascii_case("header") {
                    Ok(Some(HttpAuthPlace::Header))
                } else if value.eq_ignore_ascii_case("url") {
                    Ok(Some(HttpAuthPlace::Url))
                } else {
                    match value.as_str() {
                        "undefined" | "" => Ok(None),
                        other => Err(format!(
                            "Auth Placement must be 'Header', 'Url', or undefined, got '{}'",
                            other
                        )),
                    }
                }
            }
            _ => Err("Auth Placement must be a string or undefined".to_string()),
        }
    }
}

#[cfg(test)]
fn null_value() -> Value {
    Value {
        kind: Some(Kind::NullValue(0)),
    }
}

fn headers_from_value(value: &Value) -> Result<Struct, Signal> {
    match value.kind.as_ref() {
        Some(Kind::StructValue(headers)) => Ok(headers.clone()),
        Some(Kind::NullValue(_)) | None => Ok(Struct {
            fields: HashMap::new(),
        }),
        _ => Err(fail(
            "InvalidArgumentRuntimeError",
            "Headers must be an object or undefined",
        )),
    }
}

fn send_request(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args =>
        http_method: String,
        url: String,
        http_auth: Value,
        http_auth_value: Value,
        http_auth_place: Value,
        http_schema: String,
        payload: Value,
        headers: Value,
    );
    let mut url = url;

    let http_headers = match headers_from_value(&headers) {
        Ok(headers) => headers,
        Err(signal) => return signal,
    };

    let mut headers = match encode_headers(&http_headers) {
        Ok(headers) => headers,
        Err(message) => return fail("InvalidArgumentRuntimeError", message),
    };

    let auth = match HttpAuthType::from_value(&http_auth) {
        Ok(auth) => auth,
        Err(message) => return fail("InvalidArgumentRuntimeError", message),
    };
    let auth_place = match HttpAuthPlace::from_value(&http_auth_place) {
        Ok(auth_place) => auth_place,
        Err(message) => return fail("InvalidArgumentRuntimeError", message),
    };
    if let Err(message) = apply_auth(&auth, &http_auth_value, auth_place, &mut headers, &mut url) {
        return fail("InvalidArgumentRuntimeError", message);
    }

    let request_body = match encode_request_payload(&payload, &http_schema) {
        Ok(result) => result,
        Err(message) => return fail("InvalidArgumentRuntimeError", message),
    };

    insert_header(&mut headers, "content-type", http_schema);

    let http_method = match http::Method::from_bytes(http_method.as_bytes()) {
        Ok(value) => value,
        Err(_) => {
            return fail(
                "InvalidArgumentRuntimeError",
                format!("Invalid HTTP method '{}'", http_method),
            );
        }
    };

    let mut request_builder = http::Request::builder().method(http_method).uri(&url);
    for (name, value) in &headers {
        request_builder = request_builder.header(name, value);
    }

    let response_result = match request_body {
        Some(bytes) => {
            let request = match request_builder.body(bytes) {
                Ok(request) => request,
                Err(err) => {
                    return fail(
                        "InvalidArgumentRuntimeError",
                        format!("Invalid HTTP request: {}", err),
                    );
                }
            };
            request
                .with_default_agent()
                .configure()
                .http_status_as_error(false)
                .allow_non_standard_methods(true)
                .run()
        }
        None => {
            let request = match request_builder.body(()) {
                Ok(request) => request,
                Err(err) => {
                    return fail(
                        "InvalidArgumentRuntimeError",
                        format!("Invalid HTTP request: {}", err),
                    );
                }
            };
            request
                .with_default_agent()
                .configure()
                .http_status_as_error(false)
                .allow_non_standard_methods(true)
                .run()
        }
    };

    let response = match response_result {
        Ok(response) => response,
        Err(err) => {
            return fail(
                "HttpRequestRuntimeError",
                format!("HTTP request error while sending request: {}", err),
            );
        }
    };

    let status_code = response.status().as_u16() as i64;
    let response_headers = decode_headers(&response);
    let response_payload = match decode_response_payload(response) {
        Ok(result) => result,
        Err(message) => return fail("HttpRequestRuntimeError", message),
    };

    let mut fields = HashMap::new();
    fields.insert("http_status_code".to_string(), status_code.to_value());
    fields.insert(
        "headers".to_string(),
        Value {
            kind: Some(Kind::StructValue(response_headers)),
        },
    );
    fields.insert("payload".to_string(), response_payload);

    Signal::Success(Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    })
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

fn insert_header(headers: &mut HashMap<String, String>, name: &str, value: String) {
    if let Some(existing_name) = headers
        .keys()
        .find(|existing_name| existing_name.eq_ignore_ascii_case(name))
        .cloned()
    {
        headers.insert(existing_name, value);
    } else {
        headers.insert(name.to_string(), value);
    }
}

fn apply_auth(
    auth: &HttpAuthType,
    auth_value: &Value,
    auth_place: Option<HttpAuthPlace>,
    headers: &mut HashMap<String, String>,
    url: &mut String,
) -> Result<(), String> {
    let Some(place) = auth_place.or(match auth {
        HttpAuthType::None => None,
        _ => Some(HttpAuthPlace::Header),
    }) else {
        return Ok(());
    };

    match auth {
        HttpAuthType::None => Ok(()),
        HttpAuthType::Bearer => {
            if place != HttpAuthPlace::Header {
                return Err("Bearer auth must use Header placement".to_string());
            }
            let token = auth_string_value(auth_value, "Bearer auth value")?;
            insert_header(headers, "authorization", format!("Bearer {}", token));
            Ok(())
        }
        HttpAuthType::Basic => {
            if place != HttpAuthPlace::Header {
                return Err("Basic auth must use Header placement".to_string());
            }
            let (username, password) = basic_auth_credentials(auth_value)?;
            let encoded =
                base64::prelude::BASE64_STANDARD.encode(format!("{}:{}", username, password));
            insert_header(headers, "authorization", format!("Basic {}", encoded));
            Ok(())
        }
        HttpAuthType::XApiKey => {
            let key = auth_string_value(auth_value, "X-API-Key auth value")?;
            match place {
                HttpAuthPlace::Header => insert_header(headers, "X-API-Key", key),
                HttpAuthPlace::Url => append_query_param(url, "X-API-Key", &key),
            }
            Ok(())
        }
        HttpAuthType::Custom(scheme) => {
            let value = auth_string_value(auth_value, "Custom auth value")?;
            match place {
                HttpAuthPlace::Header => insert_header(headers, "authorization", value),
                HttpAuthPlace::Url => append_query_param(url, scheme, &value),
            }
            Ok(())
        }
    }
}

fn auth_string_value(value: &Value, label: &str) -> Result<String, String> {
    match value.kind.as_ref() {
        Some(Kind::StringValue(value)) => Ok(value.clone()),
        Some(Kind::NullValue(_)) | None => Err(format!("{} must be provided", label)),
        _ => Err(format!("{} must be a string", label)),
    }
}

fn basic_auth_credentials(value: &Value) -> Result<(String, String), String> {
    let Some(Kind::StructValue(credentials)) = value.kind.as_ref() else {
        return Err("Basic auth value must be an object with username and password".to_string());
    };

    let username = credentials
        .fields
        .get("username")
        .ok_or_else(|| "Basic auth value is missing username".to_string())
        .and_then(|value| auth_string_value(value, "Basic auth username"))?;
    let password = credentials
        .fields
        .get("password")
        .ok_or_else(|| "Basic auth value is missing password".to_string())
        .and_then(|value| auth_string_value(value, "Basic auth password"))?;
    Ok((username, password))
}

fn append_query_param(url: &mut String, name: &str, value: &str) {
    let fragment = url.find('#').map(|index| url.split_off(index));
    let separator = if url.contains('?') { '&' } else { '?' };
    url.push(separator);
    url.push_str(&percent_encode_query(name));
    url.push('=');
    url.push_str(&percent_encode_query(value));
    if let Some(fragment) = fragment {
        url.push_str(&fragment);
    }
}

fn percent_encode_query(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    encoded
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

fn normalize_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn encode_request_payload(payload: &Value, content_type: &str) -> Result<Option<Vec<u8>>, String> {
    if matches!(payload.kind.as_ref(), Some(Kind::NullValue(_)) | None) {
        return Ok(None);
    }

    let format = format_for_content_type(content_type)?;
    let protobuf = serde_json::to_vec(payload)
        .map_err(|err| format!("Unable to serialize protobuf request payload: {err}"))?;
    let engine = ConversionEngine::with_default_codecs();
    let body = engine
        .convert(
            &protobuf,
            Format::Protobuf,
            format,
            &DecodeContext,
            &EncodeContext::default(),
        )
        .map_err(|err| {
            format!(
                "Unable to convert request payload to '{}': {err}",
                content_type
            )
        })?;
    Ok(Some(body))
}

fn format_for_content_type(content_type: &str) -> Result<Format, String> {
    let normalized = normalize_content_type(content_type);
    let format = match normalized.as_str() {
        "application/json" | "text/json" => Format::Json,
        value if value.ends_with("+json") => Format::Json,
        "application/xhtml+xml" => Format::Html,
        "application/xml" | "text/xml" => Format::Xml,
        value if value.ends_with("+xml") => Format::Xml,
        "text/html" => Format::Html,
        "text/plain" => Format::Text,
        "text/csv" | "application/csv" => Format::Csv,
        "application/x-www-form-urlencoded" => Format::HttpForm,
        _ => {
            return Err(format!(
                "Unsupported content-type '{}' for http::request::send",
                content_type
            ));
        }
    };
    Ok(format)
}

fn decode_headers(response: &http::Response<Body>) -> Struct {
    let mut fields = HashMap::new();
    for (name, value) in response.headers().iter() {
        if let Ok(value) = value.to_str() {
            fields.insert(name.as_str().to_string(), value.to_string().to_value());
        }
    }
    Struct { fields }
}

fn decode_response_payload(response: http::Response<Body>) -> Result<Value, String> {
    let content_type = response
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase());

    let mut bytes = Vec::new();
    let (_, body) = response.into_parts();
    let mut reader = body.into_reader();
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
            && let Ok(json) = serde_json::from_str::<JsonValue>(&text)
        {
            return Ok(from_json_value(json));
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

    fn encoded_body(payload: &Value, content_type: &str) -> Vec<u8> {
        encode_request_payload(payload, content_type)
            .unwrap_or_else(|err| panic!("unable to encode {content_type} payload: {err}"))
            .unwrap_or_default()
    }

    #[test]
    fn encode_request_payload_serializes_every_non_null_value_kind_to_json() {
        let cases = [
            (serde_json::json!(true), "true"),
            (serde_json::json!(42), "42"),
            (serde_json::json!(1.5), "1.5"),
            (serde_json::json!("hello"), r#""hello""#),
            (serde_json::json!(["hello", 42]), r#"["hello",42]"#),
            (
                serde_json::json!({"active": true, "name": "Tom"}),
                r#"{"active":true,"name":"Tom"}"#,
            ),
        ];

        for (input, expected) in cases {
            let payload = from_json_value(input);
            assert_eq!(
                encoded_body(&payload, "application/json"),
                expected.as_bytes()
            );
        }
    }

    #[test]
    fn encode_request_payload_omits_null_and_missing_values() {
        for payload in [
            Value {
                kind: Some(Kind::NullValue(0)),
            },
            Value { kind: None },
        ] {
            assert_eq!(
                encode_request_payload(&payload, "application/json").unwrap_or(Some(vec![1])),
                None
            );
        }
    }

    #[test]
    fn encode_request_payload_does_not_detect_formats_inside_strings() {
        let text_body = encode_request_payload(&string_value("hello"), "text/plain; charset=utf-8")
            .unwrap_or(None);

        let body = text_body.unwrap_or_default();
        assert_eq!(body, b"hello");

        let json_body =
            encode_request_payload(&string_value(r#"{"user":"tom"}"#), "application/json")
                .unwrap_or(None);
        assert_eq!(json_body.unwrap_or_default(), br#""{\"user\":\"tom\"}""#);

        let xml_body =
            encode_request_payload(&string_value("<ok />"), "application/xml").unwrap_or(None);
        assert_eq!(xml_body.unwrap_or_default(), b"<data>&lt;ok /&gt;</data>");
    }

    #[test]
    fn encode_request_payload_supports_every_http_target_format() {
        let object = from_json_value(serde_json::json!({
            "user": {
                "name": "Tom"
            }
        }));
        assert_eq!(
            encoded_body(&object, "application/json"),
            br#"{"user":{"name":"Tom"}}"#
        );
        assert_eq!(
            encoded_body(&object, "application/xml"),
            b"<user><name>Tom</name></user>"
        );
        assert_eq!(
            encoded_body(&object, "text/html"),
            b"<user><name>Tom</name></user>"
        );
        assert_eq!(
            encoded_body(&string_value("hello world"), "text/plain"),
            b"hello world"
        );

        let rows = from_json_value(serde_json::json!([
            {"name": "Tom", "role": "admin"},
            {"name": "Ada", "role": "user"}
        ]));
        assert_eq!(
            encoded_body(&rows, "text/csv"),
            b"name,role\nTom,admin\nAda,user\n"
        );

        let form = from_json_value(serde_json::json!({
            "email": "tom@example.com",
            "name": "Tom Doe"
        }));
        assert_eq!(
            encoded_body(&form, "application/x-www-form-urlencoded"),
            b"email=tom%40example.com&name=Tom+Doe"
        );
    }

    #[test]
    fn format_for_content_type_supports_parameters_and_standard_aliases() {
        let cases = [
            ("application/json; charset=utf-8", Format::Json),
            ("text/json", Format::Json),
            ("application/problem+json", Format::Json),
            ("application/xml; charset=utf-8", Format::Xml),
            ("text/xml", Format::Xml),
            ("application/atom+xml", Format::Xml),
            ("text/html", Format::Html),
            ("application/xhtml+xml", Format::Html),
            ("text/plain; charset=utf-8", Format::Text),
            ("text/csv", Format::Csv),
            ("application/csv", Format::Csv),
            ("application/x-www-form-urlencoded", Format::HttpForm),
        ];

        for (content_type, expected) in cases {
            assert_eq!(
                format_for_content_type(content_type),
                Ok(expected),
                "unexpected mapping for {content_type}"
            );
        }
    }

    #[test]
    fn encode_request_payload_rejects_unsupported_content_types() {
        let err = encode_request_payload(&string_value("hello"), "application/octet-stream");
        let Err(err) = err else {
            panic!("expected unsupported content-type error");
        };
        assert!(err.contains("Unsupported content-type"));
    }

    #[test]
    fn encode_request_payload_propagates_information_loss_errors() {
        let invalid_csv = from_json_value(serde_json::json!({
            "name": "Tom"
        }));
        let csv_error = encode_request_payload(&invalid_csv, "text/csv")
            .expect_err("object should not encode as CSV rows");
        assert!(csv_error.contains("conversion would lose information"));
        assert!(csv_error.contains("top-level array"));

        let invalid_form = from_json_value(serde_json::json!({
            "active": true
        }));
        let form_error = encode_request_payload(&invalid_form, "application/x-www-form-urlencoded")
            .expect_err("typed form value should not be coerced to a string");
        assert!(form_error.contains("conversion would lose information"));
        assert!(form_error.contains("form fields must be strings"));
    }

    #[test]
    fn apply_auth_maps_supported_auth_variants() {
        let mut headers = HashMap::new();
        let mut url = "https://example.test/resource".to_string();
        apply_auth(
            &HttpAuthType::Bearer,
            &string_value("token"),
            Some(HttpAuthPlace::Header),
            &mut headers,
            &mut url,
        )
        .unwrap_or_else(|err| panic!("bearer auth failed: {}", err));
        assert_eq!(
            headers.get("authorization").map(String::as_str),
            Some("Bearer token")
        );

        let basic_value = Value {
            kind: Some(Kind::StructValue(Struct {
                fields: HashMap::from([
                    ("username".to_string(), string_value("u")),
                    ("password".to_string(), string_value("p")),
                ]),
            })),
        };
        apply_auth(
            &HttpAuthType::Basic,
            &basic_value,
            Some(HttpAuthPlace::Header),
            &mut headers,
            &mut url,
        )
        .unwrap_or_else(|err| panic!("basic auth failed: {}", err));
        assert_eq!(
            headers.get("authorization").map(String::as_str),
            Some("Basic dTpw")
        );

        apply_auth(
            &HttpAuthType::XApiKey,
            &string_value("a b"),
            Some(HttpAuthPlace::Url),
            &mut headers,
            &mut url,
        )
        .unwrap_or_else(|err| panic!("api key auth failed: {}", err));
        assert_eq!(url, "https://example.test/resource?X-API-Key=a%20b");
    }

    #[test]
    fn auth_type_parses_builtin_values_case_insensitively() {
        assert_eq!(
            HttpAuthType::from_value(&string_value("bearer")),
            Ok(HttpAuthType::Bearer)
        );
        assert_eq!(
            HttpAuthType::from_value(&string_value("basic")),
            Ok(HttpAuthType::Basic)
        );
        assert_eq!(
            HttpAuthType::from_value(&string_value("x-api-key")),
            Ok(HttpAuthType::XApiKey)
        );
    }

    #[test]
    fn auth_place_parses_values_case_insensitively() {
        assert_eq!(
            HttpAuthPlace::from_value(&string_value("header")),
            Ok(Some(HttpAuthPlace::Header))
        );
        assert_eq!(
            HttpAuthPlace::from_value(&string_value("url")),
            Ok(Some(HttpAuthPlace::Url))
        );
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
            fields: HashMap::from([
                ("x-bool".to_string(), true.to_value()),
                ("Content-Type".to_string(), string_value("text/plain")),
            ]),
        };
        let args = vec![
            Argument::Eval(string_value("POST")),
            Argument::Eval(string_value(&format!("http://{}/echo?x=1", addr))),
            Argument::Eval(null_value()),
            Argument::Eval(null_value()),
            Argument::Eval(null_value()),
            Argument::Eval(string_value("application/json")),
            Argument::Eval(request_payload),
            Argument::Eval(Value {
                kind: Some(Kind::StructValue(request_headers)),
            }),
        ];
        let mut ctx = ValueStore::default();
        let mut run = |_: &crate::handler::argument::Thunk, _: &mut ValueStore| Signal::Stop;

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
