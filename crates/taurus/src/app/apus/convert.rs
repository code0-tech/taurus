//! Conversions between Taurus/Tucana values and the Apus wire encoding.
//! Encoding writes straight from `tucana::shared::Value` into the frame
//! buffer; decoding builds `Value`s straight from the frame bytes.

use std::collections::HashMap;

use apus_protocol::value::tag;
use apus_protocol::{ApusValue, DecodeError, ErrorInfo, Reader, StreamOut, Writer, limits};
use tucana::aquila::{ActionNodeSubFlowValue, ActionNodeValue, action_node_value};
use tucana::shared::number_value::Number;
use tucana::shared::value::Kind;
use tucana::shared::{
    Error as TucanaError, ListValue, NumberValue, Struct, SubFlowFunction, SubFlowFunctionResult,
    Value,
};

/// Large strings go out as streams when `streams` is set (automatic streaming).
pub type Streams<'a, 'b, 'c> = &'a mut Option<&'b mut StreamOut<'c>>;

pub fn write_value(w: &mut Writer, value: &Value, streams: Streams<'_, '_, '_>) {
    match &value.kind {
        None | Some(Kind::NullValue(_)) => w.null(),
        Some(Kind::BoolValue(b)) => w.bool(*b),
        Some(Kind::NumberValue(NumberValue { number })) => match number {
            Some(Number::Integer(i)) => w.int(*i),
            Some(Number::Float(f)) => w.float(*f),
            None => w.null(),
        },
        Some(Kind::StringValue(s)) => w.string_or_stream(s, streams.as_deref_mut()),
        Some(Kind::ListValue(list)) => {
            w.list_header(list.values.len());
            for item in &list.values {
                write_value(w, item, streams);
            }
        }
        Some(Kind::StructValue(st)) => write_struct(w, st, streams),
    }
}

fn write_struct(w: &mut Writer, st: &Struct, streams: Streams<'_, '_, '_>) {
    w.object_header(st.fields.len());
    for (key, item) in &st.fields {
        w.key(key);
        write_value(w, item, streams);
    }
}

fn write_schema(w: &mut Writer, schema: &Option<Struct>) {
    match schema {
        Some(st) => write_struct(w, st, &mut None),
        None => w.null(),
    }
}

/// Writes one `CALL` parameter. A literal without references is a plain value;
/// a literal with inline `${…}` references is forwarded as a template (the
/// action resolves it, as with 0.0.82); a sub-flow becomes a handle.
pub fn write_param(w: &mut Writer, node: &ActionNodeValue, streams: Streams<'_, '_, '_>) {
    match &node.value {
        Some(action_node_value::Value::LiteralValue(literal)) => {
            if literal.references.is_empty() {
                match &literal.value {
                    Some(value) => write_value(w, value, streams),
                    None => w.null(),
                }
                return;
            }
            w.template_header();
            match &literal.value {
                Some(value) => write_value(w, value, streams),
                None => w.null(),
            }
            w.u16(literal.references.len() as u16);
            for reference in &literal.references {
                w.str(&reference.signature);
                match &reference.value {
                    Some(value) => write_param(w, value, streams),
                    None => w.null(),
                }
            }
        }
        Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
            execution_identifier,
            input_schema,
            output_schema,
        })) => {
            w.u8(tag::SUB_FLOW);
            w.str(execution_identifier);
            write_schema(w, input_schema);
            write_schema(w, output_schema);
        }
        None => w.null(),
    }
}

/// Placeholder for a stream inside a decoded value, replaced by the stream's
/// content once it has been received (see [`fill_streams`]).
const STREAM_MARKER: &str = "\u{0}apus-stream:";

/// Reads a value sent by an action. Stream values become placeholders; their
/// `(id, length hint)` are pushed to `streams` so the caller can collect them
/// before using the value. Hardened like `apus_protocol::Reader::value`: depth
/// limit, counts checked against the remaining bytes, unique keys, valid UTF-8.
pub fn read_value(r: &mut Reader<'_>, streams: &mut Vec<(u32, i64)>) -> Result<Value, DecodeError> {
    read_at(r, streams, 0)
}

fn read_at(
    r: &mut Reader<'_>,
    streams: &mut Vec<(u32, i64)>,
    depth: usize,
) -> Result<Value, DecodeError> {
    if depth > limits::MAX_DEPTH {
        return Err(DecodeError::TooDeep);
    }
    let kind = match r.u8()? {
        tag::NULL => Kind::NullValue(0),
        tag::FALSE => Kind::BoolValue(false),
        tag::TRUE => Kind::BoolValue(true),
        tag::INT => Kind::NumberValue(NumberValue {
            number: Some(Number::Integer(r.i64()?)),
        }),
        tag::FLOAT => Kind::NumberValue(NumberValue {
            number: Some(Number::Float(r.f64()?)),
        }),
        tag::STRING => {
            let text = r.str()?;
            // The placeholder prefix is reserved: a literal string must never be
            // mistaken for (and replaced by) a stream's content.
            if text.starts_with(STREAM_MARKER) {
                return Err(DecodeError::Invalid("reserved string prefix"));
            }
            Kind::StringValue(text.to_owned())
        }
        tag::LIST => {
            let count = r.count(1)?;
            let mut values = Vec::with_capacity(count.min(1024));
            for _ in 0..count {
                values.push(read_at(r, streams, depth + 1)?);
            }
            Kind::ListValue(ListValue { values })
        }
        tag::OBJECT => {
            let count = r.count(5)?;
            let mut fields = HashMap::with_capacity(count.min(1024));
            for _ in 0..count {
                let key = r.str()?.to_owned();
                let value = read_at(r, streams, depth + 1)?;
                if fields.insert(key, value).is_some() {
                    return Err(DecodeError::DuplicateKey);
                }
            }
            Kind::StructValue(Struct { fields })
        }
        tag::STREAM => {
            let id = r.u32()?;
            let _content_type = r.str()?;
            let length = r.i64()?;
            streams.push((id, length));
            Kind::StringValue(format!("{STREAM_MARKER}{id}"))
        }
        other => return Err(DecodeError::InvalidTag(other)),
    };
    Ok(Value { kind: Some(kind) })
}

/// Reads a function an action returned as the whole value of a `RETURN`
/// (tucana `SubFlowFunctionResult`); the `FUNCTION` tag is already consumed.
/// Schemas are objects or null and may not contain streams.
pub fn read_function(r: &mut Reader<'_>) -> Result<SubFlowFunctionResult, DecodeError> {
    let function_identifier = r.str()?.to_owned();
    let definition_source = r.str()?.to_owned();
    let mut streams = Vec::new();
    let input_schema = schema(read_at(r, &mut streams, 1)?)?;
    let output_schema = schema(read_at(r, &mut streams, 1)?)?;
    if !streams.is_empty() {
        return Err(DecodeError::Invalid("stream in a function schema"));
    }
    Ok(SubFlowFunctionResult {
        function: Some(SubFlowFunction {
            function_identifier,
            definition_source: (!definition_source.is_empty()).then_some(definition_source),
        }),
        input_schema,
        output_schema,
        name: r.str()?.to_owned(),
        description: r.str()?.to_owned(),
    })
}

fn schema(value: Value) -> Result<Option<Struct>, DecodeError> {
    match value.kind {
        None | Some(Kind::NullValue(_)) => Ok(None),
        Some(Kind::StructValue(schema)) => Ok(Some(schema)),
        Some(_) => Err(DecodeError::Invalid(
            "function schema must be an object or null",
        )),
    }
}

/// Replaces stream placeholders with the received bytes. Flow values have no
/// bytes type, so a stream stands for a string and must be valid UTF-8.
pub fn fill_streams(value: &mut Value, contents: &mut HashMap<u32, Vec<u8>>) -> Result<(), String> {
    match &mut value.kind {
        Some(Kind::StringValue(s)) => {
            if let Some(id) = s
                .strip_prefix(STREAM_MARKER)
                .and_then(|id| id.parse::<u32>().ok())
            {
                let bytes = contents
                    .remove(&id)
                    .ok_or_else(|| format!("stream {id} is not part of this value"))?;
                *s = String::from_utf8(bytes)
                    .map_err(|_| format!("stream {id} is not valid UTF-8 text"))?;
            }
            Ok(())
        }
        Some(Kind::ListValue(list)) => list
            .values
            .iter_mut()
            .try_for_each(|v| fill_streams(v, contents)),
        Some(Kind::StructValue(st)) => st
            .fields
            .values_mut()
            .try_for_each(|v| fill_streams(v, contents)),
        _ => Ok(()),
    }
}

pub fn from_apus(value: &ApusValue) -> Value {
    let kind = match value {
        ApusValue::Null => Kind::NullValue(0),
        ApusValue::Bool(b) => Kind::BoolValue(*b),
        ApusValue::Int(i) => Kind::NumberValue(NumberValue {
            number: Some(Number::Integer(*i)),
        }),
        ApusValue::Float(f) => Kind::NumberValue(NumberValue {
            number: Some(Number::Float(*f)),
        }),
        ApusValue::String(s) => Kind::StringValue(s.clone()),
        ApusValue::List(items) => Kind::ListValue(ListValue {
            values: items.iter().map(from_apus).collect(),
        }),
        ApusValue::Object(fields) => Kind::StructValue(Struct {
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), from_apus(v)))
                .collect(),
        }),
        // Params, streams and returned functions never reach the engine as such.
        ApusValue::Template { .. }
        | ApusValue::SubFlow { .. }
        | ApusValue::Stream { .. }
        | ApusValue::Function { .. } => Kind::NullValue(0),
    };
    Value { kind: Some(kind) }
}

/// Apus error (`ERROR`, `EXEC_ERROR`, `SUBFLOW_ERROR`) -> `shared.Error`.
pub fn to_tucana_error(error: ErrorInfo) -> TucanaError {
    TucanaError {
        code: error.code,
        category: error.category,
        message: error.message,
        timestamp: error.timestamp_ms,
        version: error.version,
        dependencies: error.dependencies.into_iter().collect(),
        details: match &error.details {
            ApusValue::Object(_) => match from_apus(&error.details).kind {
                Some(Kind::StructValue(st)) => Some(st),
                _ => None,
            },
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucana::aquila::{ActionInlineReferenceValue, ActionLiteralValue};

    fn string(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.into())),
        }
    }

    fn int(i: i64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(NumberValue {
                number: Some(Number::Integer(i)),
            })),
        }
    }

    #[test]
    fn value_round_trips() {
        let value = Value {
            kind: Some(Kind::StructValue(Struct {
                fields: HashMap::from([
                    ("a".to_string(), int(7)),
                    (
                        "b".to_string(),
                        Value {
                            kind: Some(Kind::ListValue(ListValue {
                                values: vec![
                                    string("x"),
                                    Value {
                                        kind: Some(Kind::BoolValue(true)),
                                    },
                                ],
                            })),
                        },
                    ),
                ]),
            })),
        };
        let mut w = Writer::default();
        write_value(&mut w, &value, &mut None);
        let bytes = w.into_inner();
        let decoded = read_value(&mut Reader::new(&bytes), &mut Vec::new()).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn templates_and_sub_flows_are_forwarded_as_params() {
        let template = ActionNodeValue {
            value: Some(action_node_value::Value::LiteralValue(ActionLiteralValue {
                value: Some(string("id-${n}")),
                references: vec![ActionInlineReferenceValue {
                    signature: "n".into(),
                    value: Some(ActionNodeValue {
                        value: Some(action_node_value::Value::LiteralValue(ActionLiteralValue {
                            value: Some(int(5)),
                            references: vec![],
                        })),
                    }),
                }],
            })),
        };
        let sub_flow = ActionNodeValue {
            value: Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                execution_identifier: "sf-1".into(),
                input_schema: None,
                output_schema: None,
            })),
        };
        let mut w = Writer::default();
        write_param(&mut w, &template, &mut None);
        write_param(&mut w, &sub_flow, &mut None);
        let bytes = w.into_inner();
        let mut r = Reader::new(&bytes);
        let template = r.value().unwrap();
        assert_eq!(
            apus_protocol::template::resolve(template).unwrap(),
            ApusValue::string("id-5")
        );
        assert_eq!(
            r.value().unwrap(),
            ApusValue::SubFlow {
                id: "sf-1".into(),
                input_schema: Box::new(ApusValue::Null),
                output_schema: Box::new(ApusValue::Null),
            }
        );
    }

    #[test]
    fn hostile_values_are_rejected() {
        let mut deep = Vec::new();
        for _ in 0..1000 {
            deep.push(tag::LIST);
            deep.extend_from_slice(&1u32.to_le_bytes());
        }
        deep.push(tag::NULL);
        assert_eq!(
            read_value(&mut Reader::new(&deep), &mut Vec::new()),
            Err(DecodeError::TooDeep)
        );

        let mut huge = vec![tag::OBJECT];
        huge.extend_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            read_value(&mut Reader::new(&huge), &mut Vec::new()),
            Err(DecodeError::CountTooLarge)
        );

        let mut w = Writer::default();
        w.object_header(2);
        w.key("a");
        w.null();
        w.key("a");
        w.int(1);
        assert_eq!(
            read_value(&mut Reader::new(&w.into_inner()), &mut Vec::new()),
            Err(DecodeError::DuplicateKey)
        );
    }

    #[test]
    fn literal_placeholders_and_invalid_text_are_rejected() {
        let mut w = Writer::default();
        w.string(&format!("{STREAM_MARKER}3"));
        assert!(read_value(&mut Reader::new(&w.into_inner()), &mut Vec::new()).is_err());

        let mut w = Writer::default();
        w.stream(3, "text/plain", 2);
        let mut value = read_value(&mut Reader::new(&w.into_inner()), &mut Vec::new()).unwrap();
        assert!(fill_streams(&mut value, &mut HashMap::from([(3, vec![0xc3, 0x28])])).is_err());
    }

    #[test]
    fn large_strings_are_streamed() {
        let next = std::sync::atomic::AtomicU32::new(2);
        let mut out = StreamOut::new(4, &next);
        let mut w = Writer::default();
        write_value(&mut w, &string("a long one"), &mut Some(&mut out));
        assert_eq!(out.pending, vec![(2, b"a long one".to_vec())]);
        let mut streams = Vec::new();
        read_value(&mut Reader::new(&w.into_inner()), &mut streams).unwrap();
        assert_eq!(streams, vec![(2, 10)]);
    }

    #[test]
    fn streams_become_placeholders_until_filled() {
        let mut w = Writer::default();
        w.list_header(2);
        w.int(1);
        w.stream(3, "text/plain", 5);
        let bytes = w.into_inner();
        let mut streams = Vec::new();
        let mut value = read_value(&mut Reader::new(&bytes), &mut streams).unwrap();
        assert_eq!(streams, vec![(3, 5)]);
        fill_streams(&mut value, &mut HashMap::from([(3, b"hello".to_vec())])).unwrap();
        let Some(Kind::ListValue(list)) = value.kind else {
            panic!("expected list")
        };
        assert_eq!(list.values, vec![int(1), string("hello")]);
    }
}
