use crate::debug::trace::{ArgKind, EdgeKind, ExecFrame, Outcome, TraceRun};
use std::collections::HashMap;

fn short(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("{}…", &s[..max])
}

fn ms(f: &ExecFrame) -> Option<u128> {
    f.end.map(|e| e.duration_since(f.start).as_millis())
}

pub fn render_trace(run: &TraceRun) -> String {
    let mut by_id: HashMap<u64, &ExecFrame> = HashMap::new();
    for f in &run.frames {
        by_id.insert(f.frame_id, f);
    }

    let mut out = String::new();
    render_frame(run.root, &by_id, "", true, &mut out);
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

    let dur = ms(f).map(|m| format!(" ({}ms)", m)).unwrap_or_default();

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
            ""
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
            short(&a.preview, 1600),
            prefix = prefix,
            pfx = pfx
        ));
    }

    // outcome
    let outcome_line = match &f.outcome {
        Some(Outcome::Success { value_preview }) => format!("✅ {}", short(value_preview, 1800)),
        Some(Outcome::Failure { error_preview }) => format!("❌ {}", short(error_preview, 1800)),
        Some(Outcome::Return { value_preview }) => format!("↩️  {}", short(value_preview, 1800)),
        Some(Outcome::Respond { value_preview }) => format!("💬 {}", short(value_preview, 1800)),
        Some(Outcome::Stop) => "🛑 Stop".to_string(),
        None => "…".to_string(),
    };

    let pfx = if prefix.is_empty() {
        ""
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

    let new_prefix = if prefix.is_empty() {
        String::new()
    } else {
        format!("{}{}", prefix, if is_last { "   " } else { "│  " })
    };

    for (idx, (edge, child_id)) in kids.iter().enumerate() {
        let last = idx + 1 == kids.len();

        // print edge label line (nice readability)
        let edge_label = match edge {
            EdgeKind::Next => "→ NEXT".to_string(),
            EdgeKind::EagerCall { arg_index } => format!("↳ eager(arg#{})", arg_index),
        };

        let edge_pfx = if prefix.is_empty() {
            ""
        } else if is_last {
            "   "
        } else {
            "│  "
        };

        out.push_str(&format!(
            "{prefix}{edge_pfx}   {edge}\n",
            prefix = prefix,
            edge_pfx = edge_pfx,
            edge = edge_label
        ));

        render_frame(*child_id, by_id, &new_prefix, last, out);
    }
}
