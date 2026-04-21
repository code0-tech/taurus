//! Runtime execution trace model.
//!
//! This model captures node frames, argument resolution, control-flow edges,
//! and final outcomes for one execution run.

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

/// One executed node invocation.
#[derive(Debug, Clone)]
pub struct ExecFrame {
    pub frame_id: u64,
    pub node_id: i64,
    pub function_name: String,
    pub args: Vec<ArgTrace>,
    pub outcome: Option<Outcome>,
    pub start: Instant,
    pub end: Option<Instant>,
    pub children: Vec<(EdgeKind, u64)>,
}

/// Trace data for a full execution.
#[derive(Debug, Clone)]
pub struct TraceRun {
    pub frames: Vec<ExecFrame>,
    pub root: u64,
}
