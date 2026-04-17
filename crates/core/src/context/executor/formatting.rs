use super::*;

pub(super) fn preview_value(value: &Value) -> String {
    format_value_json(value)
}

fn format_value_json(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::NumberValue(v)) => match v.number {
            Some(kind) => match kind {
                tucana::shared::number_value::Number::Integer(i) => i.to_string(),
                tucana::shared::number_value::Number::Float(f) => f.to_string(),
            },
            _ => "null".to_string(),
        },
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::StringValue(v)) => format!("{:?}", v),
        Some(Kind::NullValue(_)) | None => "null".to_string(),
        Some(Kind::ListValue(list)) => {
            let mut parts = Vec::new();
            for item in list.values.iter() {
                parts.push(format_value_json(item));
            }
            format!("[{}]", parts.join(", "))
        }
        Some(Kind::StructValue(struct_value)) => {
            let mut keys: Vec<_> = struct_value.fields.keys().collect();
            keys.sort();
            let mut parts = Vec::new();
            for key in keys.iter() {
                if let Some(v) = struct_value.fields.get(*key) {
                    parts.push(format!("{:?}: {}", key, format_value_json(v)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
    }
}

pub(super) fn preview_reference(r: &tucana::shared::ReferenceValue) -> String {
    let target = match &r.target {
        Some(Target::FlowInput(_)) => "flow_input".to_string(),
        Some(Target::NodeId(id)) => format!("node({})", id),
        Some(Target::InputType(input_type)) => format!(
            "input(node={},param={},input={})",
            input_type.node_id, input_type.parameter_index, input_type.input_index
        ),
        None => "empty".to_string(),
    };

    if r.paths.is_empty() {
        target
    } else {
        format!("{}+paths({})", target, r.paths.len())
    }
}
