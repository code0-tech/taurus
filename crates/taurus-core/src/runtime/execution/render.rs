//! Human-readable trace renderer.

use std::collections::HashMap;

use crate::runtime::execution::trace::{ArgKind, EdgeKind, ExecFrame, Outcome, TraceRun};

fn frame_micros(frame: &ExecFrame) -> Option<u128> {
    frame
        .end
        .map(|end| end.duration_since(frame.start).as_micros())
}

/// Render a trace as plain-text tree output.
pub fn render_trace(run: &TraceRun) -> String {
    let mut by_id: HashMap<u64, &ExecFrame> = HashMap::new();
    for frame in &run.frames {
        by_id.insert(frame.frame_id, frame);
    }

    let mut output = String::new();
    if let Some(total_us) = total_duration_us(&run.frames) {
        output.push_str(&format!("Total: {}us\n", total_us));
    }
    render_frame(run.root, &by_id, "", true, &mut output);
    if let Some(total_us) = total_duration_us(&run.frames) {
        output.push_str(&format!("Summary: total_time={}us\n", total_us));
    }
    output
}

fn render_frame(
    id: u64,
    by_id: &HashMap<u64, &ExecFrame>,
    prefix: &str,
    is_last: bool,
    output: &mut String,
) {
    let frame = by_id[&id];

    let branch = if prefix.is_empty() {
        ""
    } else if is_last {
        "\\- "
    } else {
        "+- "
    };

    let duration = frame_micros(frame)
        .map(|us| format!(" ({}us)", us))
        .unwrap_or_default();

    output.push_str(&format!(
        "{prefix}{branch}#{frame_id}  node={node_id}  fn={function}{duration}\n",
        prefix = prefix,
        branch = branch,
        frame_id = frame.frame_id,
        node_id = frame.node_id,
        function = frame.function_name,
        duration = duration
    ));

    for arg in &frame.args {
        let continuation = if prefix.is_empty() || is_last {
            "   "
        } else {
            "|  "
        };

        let kind = match &arg.kind {
            ArgKind::Literal => "lit".to_string(),
            ArgKind::Reference { reference, hit } => {
                let suffix = if *hit { "hit" } else { "miss" };
                format!("ref({:?}, {})", reference, suffix)
            }
            ArgKind::Thunk {
                node_id,
                eager,
                executed,
            } => {
                let mode = if *eager { "eager" } else { "lazy" };
                let exec = if *executed { "executed" } else { "deferred" };
                format!("thunk(node={}, {}, {})", node_id, mode, exec)
            }
        };

        output.push_str(&format!(
            "{prefix}{continuation}   arg[{index}] {kind:<24} {preview}\n",
            prefix = prefix,
            continuation = continuation,
            index = arg.index,
            kind = kind,
            preview = arg.preview
        ));
    }

    let outcome = match &frame.outcome {
        Some(Outcome::Success { value_preview }) => format!("[SUCCESS] {}", value_preview),
        Some(Outcome::Failure { error_preview }) => format!("[FAILURE] {}", error_preview),
        Some(Outcome::Return { value_preview }) => format!("[RETURN] {}", value_preview),
        Some(Outcome::Respond { value_preview }) => format!("[RESPOND] {}", value_preview),
        Some(Outcome::Stop) => "[STOP]".to_string(),
        None => "...".to_string(),
    };

    let continuation = if prefix.is_empty() || is_last {
        "   "
    } else {
        "|  "
    };
    output.push_str(&format!(
        "{prefix}{continuation}   => {outcome}\n",
        prefix = prefix,
        continuation = continuation,
        outcome = outcome
    ));

    if frame.children.is_empty() {
        return;
    }

    let mut runtime_idx = 0usize;
    for (idx, (edge, child_id)) in frame.children.iter().enumerate() {
        let edge_last = idx + 1 == frame.children.len();
        let edge_branch = if edge_last { "\\- " } else { "+- " };

        let edge_text = match edge {
            EdgeKind::Next => "NEXT".to_string(),
            EdgeKind::EagerCall { arg_index } => format!("eager(arg#{})", arg_index),
            EdgeKind::RuntimeCall { label } => {
                runtime_idx += 1;
                match label {
                    Some(label_text) => format!("runtime(call #{}) {}", runtime_idx, label_text),
                    None => format!("runtime(call #{})", runtime_idx),
                }
            }
        };

        output.push_str(&format!(
            "{prefix}{edge_branch}{edge_text}\n",
            prefix = prefix,
            edge_branch = edge_branch,
            edge_text = edge_text
        ));

        let child_prefix = format!("{}{}", prefix, if edge_last { "   " } else { "|  " });
        render_frame(*child_id, by_id, &child_prefix, true, output);
    }
}

fn total_duration_us(frames: &[ExecFrame]) -> Option<u128> {
    let mut start = None;
    let mut end = None;

    for frame in frames {
        start = Some(match start {
            Some(current) if frame.start > current => current,
            Some(_) | None => frame.start,
        });

        if let Some(frame_end) = frame.end {
            end = Some(match end {
                Some(current) if frame_end < current => current,
                Some(_) | None => frame_end,
            });
        }
    }

    match (start, end) {
        (Some(start_time), Some(end_time)) => Some(end_time.duration_since(start_time).as_micros()),
        _ => None,
    }
}
