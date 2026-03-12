use std::time::Instant;

#[derive(Debug, Clone)]
pub enum EdgeKind {
    /// Linear control-flow via next_node_id
    Next,
    /// Eager evaluation of a thunk argument (child execution)
    EagerCall { arg_index: usize },
}

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

#[derive(Debug, Clone)]
pub struct ArgTrace {
    pub index: usize,
    pub kind: ArgKind,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Success { value_preview: String },
    Failure { error_preview: String },
    Return { value_preview: String },
    Respond { value_preview: String },
    Stop,
}

#[derive(Debug, Clone)]
pub struct ExecFrame {
    pub frame_id: u64,         // unique execution instance id
    pub node_id: i64,          // database_id
    pub function_name: String, // runtime_function_id
    pub args: Vec<ArgTrace>,
    pub outcome: Option<Outcome>,

    pub start: Instant,
    pub end: Option<Instant>,

    /// Child edges to other frames (CALLs and NEXT links)
    pub children: Vec<(EdgeKind, u64)>,
}

#[derive(Debug, Clone)]
pub struct TraceRun {
    pub frames: Vec<ExecFrame>,
    pub root: u64,
}
