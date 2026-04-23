//! Trace V2 runtime execution model.
//!
//! The model is frame-centric and stores:
//! - resolved arguments
//! - parent/child control-flow edges
//! - before/after execution store snapshots
//! - computed store diff for each frame

use std::collections::HashMap;
use std::time::Instant;

/// Relationship between two execution frames.
#[derive(Debug, Clone)]
pub enum EdgeKind {
    /// Sequential flow transition via `next_node_id`.
    Next,
    /// Eager argument child execution.
    EagerCall { arg_index: usize },
    /// Lazy runtime callback child execution.
    RuntimeCall { label: Option<String> },
}

/// Argument classification for tracing.
#[derive(Debug, Clone)]
pub enum ArgKind {
    Literal,
    Reference {
        reference: ReferenceKind,
        hit: bool,
    },
    Thunk {
        node_id: i64,
        eager: bool,
        executed: bool,
    },
}

/// Reference source kind for argument tracing.
#[derive(Debug, Clone)]
pub enum ReferenceKind {
    Result {
        node_id: i64,
    },
    InputType {
        node_id: i64,
        input_index: i64,
        parameter_index: i64,
    },
    FlowInput,
    Empty,
}

/// One traced argument on a frame.
#[derive(Debug, Clone)]
pub struct ArgTrace {
    pub index: usize,
    pub kind: ArgKind,
    pub preview: String,
}

/// Final outcome of a frame.
#[derive(Debug, Clone)]
pub enum Outcome {
    Success { value_preview: String },
    Failure { error_preview: String },
    Return { value_preview: String },
    Respond { value_preview: String },
    Stop,
}

/// One stored node result entry at snapshot time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreResultEntry {
    pub node_id: i64,
    pub status: StoreResultStatus,
    pub preview: String,
}

/// Result status in the value store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreResultStatus {
    Success,
    Error,
}

/// One temporary input slot entry at snapshot time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreInputSlotEntry {
    pub node_id: i64,
    pub parameter_index: i64,
    pub input_index: i64,
    pub preview: String,
}

/// Value store snapshot attached to a frame boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSnapshot {
    pub current_node_id: i64,
    pub flow_input_preview: String,
    pub results: Vec<StoreResultEntry>,
    pub input_slots: Vec<StoreInputSlotEntry>,
}

/// Per-frame store changes between `store_before` and `store_after`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreDiff {
    pub current_node_changed: Option<(i64, i64)>,
    pub result_sets: Vec<StoreResultEntry>,
    pub result_clears: Vec<i64>,
    pub input_slot_sets: Vec<StoreInputSlotEntry>,
    pub input_slot_clears: Vec<(i64, i64, i64)>,
}

impl StoreDiff {
    pub fn from(before: &StoreSnapshot, after: &StoreSnapshot) -> Self {
        let current_node_changed = (before.current_node_id != after.current_node_id)
            .then_some((before.current_node_id, after.current_node_id));

        let mut before_results: HashMap<i64, &StoreResultEntry> = HashMap::new();
        for entry in &before.results {
            before_results.insert(entry.node_id, entry);
        }
        let mut after_results: HashMap<i64, &StoreResultEntry> = HashMap::new();
        for entry in &after.results {
            after_results.insert(entry.node_id, entry);
        }

        let mut result_sets = Vec::new();
        for (node_id, after_entry) in &after_results {
            match before_results.get(node_id) {
                Some(before_entry)
                    if before_entry.status == after_entry.status
                        && before_entry.preview == after_entry.preview => {}
                _ => result_sets.push((*after_entry).clone()),
            }
        }
        result_sets.sort_by_key(|entry| entry.node_id);

        let mut result_clears = Vec::new();
        for node_id in before_results.keys() {
            if !after_results.contains_key(node_id) {
                result_clears.push(*node_id);
            }
        }
        result_clears.sort_unstable();

        let mut before_inputs: HashMap<(i64, i64, i64), &StoreInputSlotEntry> = HashMap::new();
        for entry in &before.input_slots {
            before_inputs.insert(
                (entry.node_id, entry.parameter_index, entry.input_index),
                entry,
            );
        }
        let mut after_inputs: HashMap<(i64, i64, i64), &StoreInputSlotEntry> = HashMap::new();
        for entry in &after.input_slots {
            after_inputs.insert(
                (entry.node_id, entry.parameter_index, entry.input_index),
                entry,
            );
        }

        let mut input_slot_sets = Vec::new();
        for (key, after_entry) in &after_inputs {
            match before_inputs.get(key) {
                Some(before_entry) if before_entry.preview == after_entry.preview => {}
                _ => input_slot_sets.push((*after_entry).clone()),
            }
        }
        input_slot_sets
            .sort_by_key(|entry| (entry.node_id, entry.parameter_index, entry.input_index));

        let mut input_slot_clears = Vec::new();
        for key in before_inputs.keys() {
            if !after_inputs.contains_key(key) {
                input_slot_clears.push(*key);
            }
        }
        input_slot_clears.sort_unstable();

        Self {
            current_node_changed,
            result_sets,
            result_clears,
            input_slot_sets,
            input_slot_clears,
        }
    }
}

/// Child link entry for one frame.
#[derive(Debug, Clone)]
pub struct FrameChild {
    pub edge: EdgeKind,
    pub child_frame_id: u64,
}

/// One executed node invocation frame.
#[derive(Debug, Clone)]
pub struct TraceFrame {
    pub frame_id: u64,
    pub parent_frame_id: Option<u64>,
    pub depth: usize,
    pub node_id: i64,
    pub function_name: String,
    pub args: Vec<ArgTrace>,
    pub outcome: Option<Outcome>,
    pub started_at: Instant,
    pub ended_at: Option<Instant>,
    pub children: Vec<FrameChild>,
    pub store_before: StoreSnapshot,
    pub store_after: Option<StoreSnapshot>,
    pub store_diff: Option<StoreDiff>,
}

/// Trace data for one full execution run.
#[derive(Debug, Clone)]
pub struct TraceRun {
    pub started_at: Instant,
    pub ended_at: Option<Instant>,
    pub root_frame_id: u64,
    pub frames: Vec<TraceFrame>,
}
