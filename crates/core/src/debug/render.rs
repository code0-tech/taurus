use crate::debug::trace::{ArgKind, EdgeKind, ExecFrame, Outcome, TraceRun};
use std::collections::HashMap;

fn micros(f: &ExecFrame) -> Option<u128> {
    f.end.map(|e| e.duration_since(f.start).as_micros())
}

pub fn render_trace(run: &TraceRun) -> String {
    let mut by_id: HashMap<u64, &ExecFrame> = HashMap::new();
    for f in &run.frames {
        by_id.insert(f.frame_id, f);
    }

    let mut out = String::new();
    if let Some(total_us) = total_duration_us(&run.frames) {
        out.push_str(&format!("Total: {}µs\n", total_us));
    }
    render_frame(run.root, &by_id, "", true, &mut out);
    if let Some(total_us) = total_duration_us(&run.frames) {
        out.push_str(&format!("Summary: total_time={}µs\n", total_us));
    }
    out
}

fn render_frame(
    id: u64,
    by_id: &HashMap<u64, &ExecFrame>,
    prefix: &str,
    is_last: bool,
    out: &mut String,
) {
    let f = by_id[&id];

    let branch = if prefix.is_empty() {
        "" // root
    } else if is_last {
        "└─ "
    } else {
        "├─ "
    };

    let dur = micros(f).map(|u| format!(" ({}µs)", u)).unwrap_or_default();

    out.push_str(&format!(
        "{prefix}{branch}#{fid}  node={nid}  fn={name}{dur}\n",
        prefix = prefix,
        branch = branch,
        fid = f.frame_id,
        nid = f.node_id,
        name = f.function_name,
        dur = dur
    ));

    // args
    for a in &f.args {
        let pfx = if prefix.is_empty() {
            "   "
        } else if is_last {
            "   "
        } else {
            "│  "
        };

        let kind = match &a.kind {
            ArgKind::Literal => "lit",
            ArgKind::Reference { hit, .. } => {
                if *hit {
                    "ref✓"
                } else {
                    "ref✗"
                }
            }
            ArgKind::Thunk {
                eager, executed, ..
            } => match (*eager, *executed) {
                (true, true) => "thunk eager✓",
                (true, false) => "thunk eager",
                (false, true) => "thunk lazy✓",
                (false, false) => "thunk lazy",
            },
        };

        out.push_str(&format!(
            "{prefix}{pfx}   arg[{}] {:<12} {}\n",
            a.index,
            kind,
            a.preview,
            prefix = prefix,
            pfx = pfx
        ));
    }

    // outcome
    let outcome_line = match &f.outcome {
        Some(Outcome::Success { value_preview }) => {
            colorize("SUCCESS", AnsiColor::Green, value_preview)
        }
        Some(Outcome::Failure { error_preview }) => {
            colorize("FAILURE", AnsiColor::Red, error_preview)
        }
        Some(Outcome::Return { value_preview }) => {
            colorize("RETURN", AnsiColor::Cyan, value_preview)
        }
        Some(Outcome::Respond { value_preview }) => {
            colorize("RESPOND", AnsiColor::Blue, value_preview)
        }
        Some(Outcome::Stop) => colorize("STOP", AnsiColor::Yellow, "Stop"),
        None => "…".to_string(),
    };

    let pfx = if prefix.is_empty() {
        "   "
    } else if is_last {
        "   "
    } else {
        "│  "
    };
    out.push_str(&format!(
        "{prefix}{pfx}   => {o}\n",
        prefix = prefix,
        pfx = pfx,
        o = outcome_line
    ));

    // children
    let kids = &f.children;
    if kids.is_empty() {
        return;
    }

    let mut runtime_idx = 0usize;
    for (idx, (edge, child_id)) in kids.iter().enumerate() {
        let last = idx + 1 == kids.len();

        // print edge label line (nice readability)
        let edge_label = match edge {
            EdgeKind::Next => "→ NEXT".to_string(),
            EdgeKind::EagerCall { arg_index } => format!("↳ eager(arg#{})", arg_index),
            EdgeKind::RuntimeCall { label } => {
                runtime_idx += 1;
                match label {
                    Some(l) => format!("↳ runtime(call #{}) {}", runtime_idx, l),
                    None => format!("↳ runtime(call #{})", runtime_idx),
                }
            }
        };

        let edge_branch = if last { "└─ " } else { "├─ " };
        out.push_str(&format!(
            "{prefix}{branch}{edge}\n",
            prefix = prefix,
            branch = edge_branch,
            edge = edge_label
        ));

        let edge_child_prefix = format!("{}{}", prefix, if last { "   " } else { "│  " });
        render_frame(*child_id, by_id, &edge_child_prefix, true, out);
    }
}

enum AnsiColor {
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
}

fn colorize(label: &str, color: AnsiColor, payload: &str) -> String {
    let code = match color {
        AnsiColor::Red => 31,
        AnsiColor::Green => 32,
        AnsiColor::Yellow => 33,
        AnsiColor::Blue => 34,
        AnsiColor::Cyan => 36,
    };
    format!("\x1b[{code}m[{label}]\x1b[0m {payload}")
}

fn total_duration_us(frames: &[ExecFrame]) -> Option<u128> {
    let mut start: Option<std::time::Instant> = None;
    let mut end: Option<std::time::Instant> = None;

    for f in frames {
        if let Some(s) = start {
            if f.start < s {
                start = Some(f.start);
            }
        } else {
            start = Some(f.start);
        }

        if let Some(f_end) = f.end {
            if let Some(e) = end {
                if f_end > e {
                    end = Some(f_end);
                }
            } else {
                end = Some(f_end);
            }
        }
    }

    match (start, end) {
        (Some(s), Some(e)) => Some(e.duration_since(s).as_micros()),
        _ => None,
    }
}
