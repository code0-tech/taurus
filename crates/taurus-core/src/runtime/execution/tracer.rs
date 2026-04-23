//! In-memory Trace V2 collector.

use std::time::Instant;

use crate::runtime::execution::trace::{
    ArgKind, ArgTrace, EdgeKind, FrameChild, Outcome, StoreDiff, StoreSnapshot, TraceFrame,
    TraceRun,
};

/// Trace collector interface used by the executor.
pub trait ExecutionTracer {
    fn enter_node(&mut self, node_id: i64, function_name: &str, store_before: StoreSnapshot)
    -> u64;
    fn record_arg(&mut self, frame_id: u64, arg: ArgTrace);
    fn link_child(&mut self, parent_frame: u64, child_frame: u64, edge: EdgeKind);
    fn mark_thunk(&mut self, frame_id: u64, arg_index: usize, eager: bool, executed: bool);
    fn mark_thunk_executed_by_node(&mut self, frame_id: u64, node_id: i64);
    fn exit_node(&mut self, frame_id: u64, outcome: Outcome, store_after: StoreSnapshot);
}

/// Default trace recorder used by the runtime engine.
pub struct Tracer {
    next_id: u64,
    run: Option<TraceRun>,
    stack: Vec<u64>,
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            run: None,
            stack: vec![],
        }
    }

    fn frames_mut(&mut self) -> &mut Vec<TraceFrame> {
        &mut self
            .run
            .as_mut()
            .expect("trace run must exist before frame mutation")
            .frames
    }

    fn get_frame_mut(&mut self, frame_id: u64) -> &mut TraceFrame {
        let idx = self
            .frames_mut()
            .iter()
            .position(|f| f.frame_id == frame_id)
            .expect("trace frame must exist");
        &mut self.frames_mut()[idx]
    }

    pub fn take_run(self) -> Option<TraceRun> {
        self.run
    }
}

impl ExecutionTracer for Tracer {
    fn enter_node(
        &mut self,
        node_id: i64,
        function_name: &str,
        store_before: StoreSnapshot,
    ) -> u64 {
        if self.run.is_none() {
            self.run = Some(TraceRun {
                started_at: Instant::now(),
                ended_at: None,
                frames: vec![],
                root_frame_id: 0,
            });
        }

        let frame_id = self.next_id;
        self.next_id += 1;
        let parent_frame_id = self.stack.last().copied();
        let depth = self.stack.len();

        let frame = TraceFrame {
            frame_id,
            parent_frame_id,
            depth,
            node_id,
            function_name: function_name.to_string(),
            args: vec![],
            outcome: None,
            started_at: Instant::now(),
            ended_at: None,
            children: vec![],
            store_before,
            store_after: None,
            store_diff: None,
        };

        let run = self
            .run
            .as_mut()
            .expect("trace run must exist before first frame");
        if run.root_frame_id == 0 {
            run.root_frame_id = frame_id;
        }
        run.frames.push(frame);

        self.stack.push(frame_id);
        frame_id
    }

    fn record_arg(&mut self, frame_id: u64, arg: ArgTrace) {
        self.get_frame_mut(frame_id).args.push(arg);
    }

    fn link_child(&mut self, parent_frame: u64, child_frame: u64, edge: EdgeKind) {
        self.get_frame_mut(parent_frame).children.push(FrameChild {
            edge,
            child_frame_id: child_frame,
        });
    }

    fn mark_thunk(&mut self, frame_id: u64, arg_index: usize, eager: bool, executed: bool) {
        let frame = self.get_frame_mut(frame_id);
        if let Some(arg) = frame.args.iter_mut().find(|a| a.index == arg_index)
            && let ArgTrace {
                kind:
                    ArgKind::Thunk {
                        eager: current_eager,
                        executed: current_executed,
                        ..
                    },
                ..
            } = arg
        {
            *current_eager = eager;
            *current_executed = executed;
        }
    }

    fn mark_thunk_executed_by_node(&mut self, frame_id: u64, node_id: i64) {
        let frame = self.get_frame_mut(frame_id);
        if let Some(arg) = frame.args.iter_mut().find(|a| {
            matches!(
                a.kind,
                ArgKind::Thunk {
                    node_id: current_node,
                    executed: false,
                    ..
                } if current_node == node_id
            )
        }) && let ArgTrace {
            kind:
                ArgKind::Thunk {
                    executed: current_executed,
                    ..
                },
            ..
        } = arg
        {
            *current_executed = true;
        }
    }

    fn exit_node(&mut self, frame_id: u64, outcome: Outcome, store_after: StoreSnapshot) {
        {
            let frame = self.get_frame_mut(frame_id);
            frame.outcome = Some(outcome);
            frame.ended_at = Some(Instant::now());
            frame.store_diff = Some(StoreDiff::from(&frame.store_before, &store_after));
            frame.store_after = Some(store_after);
        }

        let popped = self.stack.pop();
        debug_assert_eq!(popped, Some(frame_id));

        if self.stack.is_empty()
            && let Some(run) = self.run.as_mut()
        {
            run.ended_at = Some(Instant::now());
        }
    }
}
