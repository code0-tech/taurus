//! Human-readable trace renderer (single hybrid mode).

use std::collections::HashMap;

use crate::runtime::execution::trace::{
    ArgKind, EdgeKind, Outcome, StoreDiff, StoreResultStatus, TraceFrame, TraceRun,
};

struct TraceTheme;

impl TraceTheme {
    fn paint(&self, text: &str, code: &str) -> String {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    }

    fn enter(&self) -> String {
        self.paint("ENTER", "36")
    }
    fn ctx(&self) -> String {
        self.paint("CTX", "90")
    }
    fn arg(&self) -> String {
        self.paint("ARG", "94")
    }
    fn call(&self) -> String {
        self.paint("CALL", "35")
    }
    fn exit(&self) -> String {
        self.paint("EXIT", "33")
    }
    fn store(&self) -> String {
        self.paint("STORE", "32")
    }
    fn success(&self, value: &str) -> String {
        format!("{} {}", self.paint("SUCCESS", "32"), value)
    }
    fn failure(&self, value: &str) -> String {
        format!("{} {}", self.paint("FAILURE", "31"), value)
    }
    fn returned(&self, value: &str) -> String {
        format!("{} {}", self.paint("RETURN", "33"), value)
    }
    fn respond(&self, value: &str) -> String {
        format!("{} {}", self.paint("RESPOND", "35"), value)
    }
    fn stop(&self) -> String {
        self.paint("STOP", "31")
    }
}

fn frame_micros(frame: &TraceFrame) -> Option<u128> {
    frame
        .ended_at
        .map(|end| end.duration_since(frame.started_at).as_micros())
}

/// Render trace in execution order with branch markers and spacing.
pub fn render_trace(run: &TraceRun) -> String {
    let theme = TraceTheme;
    let mut by_id: HashMap<u64, &TraceFrame> = HashMap::new();
    for frame in &run.frames {
        by_id.insert(frame.frame_id, frame);
    }

    let mut out = String::new();
    if let Some(total_us) = total_duration_us(run) {
        out.push_str(&format!("Total: {}us\n", total_us));
    }

    let mut step = 1usize;
    render_frame(
        run.root_frame_id,
        &by_id,
        "",
        true,
        &theme,
        &mut step,
        &mut out,
    );

    if let Some(total_us) = total_duration_us(run) {
        out.push_str(&format!(
            "Summary: total_time={}us frames={}\n",
            total_us,
            run.frames.len()
        ));
    } else {
        out.push_str(&format!("Summary: frames={}\n", run.frames.len()));
    }
    out
}

fn render_frame(
    frame_id: u64,
    by_id: &HashMap<u64, &TraceFrame>,
    prefix: &str,
    is_last: bool,
    theme: &TraceTheme,
    step: &mut usize,
    out: &mut String,
) {
    let frame = by_id[&frame_id];
    let depth_indent = "  ".repeat(frame.depth);
    let display_prefix = format!("{}{}", depth_indent, prefix);
    let branch = if prefix.is_empty() {
        ""
    } else if is_last {
        "\\- "
    } else {
        "+- "
    };
    let continuation = if prefix.is_empty() {
        ""
    } else if is_last {
        "   "
    } else {
        "|  "
    };
    let duration = frame_micros(frame)
        .map(|us| format!(" ({}us)", us))
        .unwrap_or_default();

    if frame.depth == 0 && *step > 1 {
        out.push('\n');
    }

    out.push_str(&format!(
        "{step:04} {display_prefix}{branch}{enter:<5} #{id} node={node} fn={fn_name} depth={depth}{duration}\n",
        step = *step,
        display_prefix = display_prefix,
        branch = branch,
        enter = theme.enter(),
        id = frame.frame_id,
        node = frame.node_id,
        fn_name = frame.function_name,
        depth = frame.depth,
        duration = duration
    ));
    *step += 1;

    out.push_str(&format!(
        "{step:04} {display_prefix}{continuation}{ctx:<5} current_node={} results={} input_slots={}\n",
        frame.store_before.current_node_id,
        frame.store_before.results.len(),
        frame.store_before.input_slots.len(),
        step = *step,
        display_prefix = display_prefix,
        continuation = continuation,
        ctx = theme.ctx()
    ));
    *step += 1;

    for arg in &frame.args {
        let arg_kind = match &arg.kind {
            ArgKind::Literal => "literal".to_string(),
            ArgKind::Reference { reference, hit } => {
                let hit_state = if *hit { "hit" } else { "miss" };
                format!("reference {:?} ({})", reference, hit_state)
            }
            ArgKind::Thunk {
                node_id,
                eager,
                executed,
            } => {
                let mode = if *eager { "eager" } else { "lazy" };
                let executed_state = if *executed { "executed" } else { "deferred" };
                format!("thunk node={} {} {}", node_id, mode, executed_state)
            }
        };
        out.push_str(&format!(
            "{step:04} {display_prefix}{continuation}{arg:<5} [{index}] {kind} => {preview}\n",
            step = *step,
            display_prefix = display_prefix,
            continuation = continuation,
            arg = theme.arg(),
            index = arg.index,
            kind = arg_kind,
            preview = arg.preview
        ));
        *step += 1;
    }

    let mut runtime_idx = 0usize;
    for (idx, child) in frame.children.iter().enumerate() {
        let child_is_last = idx + 1 == frame.children.len();
        let edge_branch = if child_is_last { "\\- " } else { "+- " };
        let edge_text = match &child.edge {
            EdgeKind::Next => "next".to_string(),
            EdgeKind::EagerCall { arg_index } => format!("eager arg#{}", arg_index),
            EdgeKind::RuntimeCall { label } => {
                runtime_idx += 1;
                match label {
                    Some(label) => format!("runtime call #{} {}", runtime_idx, label),
                    None => format!("runtime call #{}", runtime_idx),
                }
            }
        };
        out.push_str(&format!(
            "{step:04} {display_prefix}{continuation}{edge_branch}{call:<5} #{parent} -> #{child} ({edge})\n",
            step = *step,
            display_prefix = display_prefix,
            continuation = continuation,
            edge_branch = edge_branch,
            call = theme.call(),
            parent = frame.frame_id,
            child = child.child_frame_id,
            edge = edge_text
        ));
        *step += 1;

        let child_prefix = format!("{}{}", prefix, continuation);
        render_frame(
            child.child_frame_id,
            by_id,
            &child_prefix,
            child_is_last,
            theme,
            step,
            out,
        );
    }

    let outcome = match &frame.outcome {
        Some(Outcome::Success { value_preview }) => theme.success(value_preview),
        Some(Outcome::Failure { error_preview }) => theme.failure(error_preview),
        Some(Outcome::Return { value_preview }) => theme.returned(value_preview),
        Some(Outcome::Respond { value_preview }) => theme.respond(value_preview),
        Some(Outcome::Stop) => theme.stop(),
        None => "INCOMPLETE".to_string(),
    };
    out.push_str(&format!(
        "{step:04} {display_prefix}{continuation}{exit:<5} #{id} signal={outcome}\n",
        step = *step,
        display_prefix = display_prefix,
        continuation = continuation,
        exit = theme.exit(),
        id = frame.frame_id,
        outcome = outcome
    ));
    *step += 1;

    if let Some(diff) = &frame.store_diff {
        render_store_diff(&display_prefix, continuation, theme, diff, step, out);
    }
}

fn render_store_diff(
    prefix: &str,
    continuation: &str,
    theme: &TraceTheme,
    diff: &StoreDiff,
    step: &mut usize,
    out: &mut String,
) {
    if let Some((from, to)) = diff.current_node_changed {
        out.push_str(&format!(
            "{step:04} {prefix}{continuation}{store:<5} current_node: {} -> {}\n",
            from,
            to,
            step = *step,
            prefix = prefix,
            continuation = continuation,
            store = theme.store()
        ));
        *step += 1;
    }

    for set in &diff.result_sets {
        let status = match set.status {
            StoreResultStatus::Success => "success",
            StoreResultStatus::Error => "error",
        };
        out.push_str(&format!(
            "{step:04} {prefix}{continuation}{store:<5} result.set node={} [{}] {}\n",
            set.node_id,
            status,
            set.preview,
            step = *step,
            prefix = prefix,
            continuation = continuation,
            store = theme.store()
        ));
        *step += 1;
    }
    for cleared in &diff.result_clears {
        out.push_str(&format!(
            "{step:04} {prefix}{continuation}{store:<5} result.clear node={}\n",
            cleared,
            step = *step,
            prefix = prefix,
            continuation = continuation,
            store = theme.store()
        ));
        *step += 1;
    }

    for set in &diff.input_slot_sets {
        out.push_str(&format!(
            "{step:04} {prefix}{continuation}{store:<5} input.set node={} param={} input={} {}\n",
            set.node_id,
            set.parameter_index,
            set.input_index,
            set.preview,
            step = *step,
            prefix = prefix,
            continuation = continuation,
            store = theme.store()
        ));
        *step += 1;
    }
    for (node_id, parameter_index, input_index) in &diff.input_slot_clears {
        out.push_str(&format!(
            "{step:04} {prefix}{continuation}{store:<5} input.clear node={} param={} input={}\n",
            node_id,
            parameter_index,
            input_index,
            step = *step,
            prefix = prefix,
            continuation = continuation,
            store = theme.store()
        ));
        *step += 1;
    }
}

fn total_duration_us(run: &TraceRun) -> Option<u128> {
    match run.ended_at {
        Some(end) => Some(end.duration_since(run.started_at).as_micros()),
        None => {
            let mut start = None;
            let mut end = None;
            for frame in &run.frames {
                start = Some(match start {
                    Some(current) if frame.started_at > current => current,
                    Some(_) | None => frame.started_at,
                });
                if let Some(frame_end) = frame.ended_at {
                    end = Some(match end {
                        Some(current) if frame_end < current => current,
                        Some(_) | None => frame_end,
                    });
                }
            }
            match (start, end) {
                (Some(start), Some(end)) => Some(end.duration_since(start).as_micros()),
                _ => None,
            }
        }
    }
}
