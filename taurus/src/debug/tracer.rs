use std::time::Instant;

use crate::debug::trace::{ArgTrace, EdgeKind, ExecFrame, Outcome, TraceRun};

pub trait ExecutionTracer {
    fn enter_node(&mut self, node_id: i64, function_name: &str) -> u64;
    fn record_arg(&mut self, frame_id: u64, arg: ArgTrace);
    fn link_child(&mut self, parent_frame: u64, child_frame: u64, edge: EdgeKind);
    fn exit_node(&mut self, frame_id: u64, outcome: Outcome);
}

pub struct Tracer {
    next_id: u64,
    pub run: Option<TraceRun>,
    stack: Vec<u64>,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            run: None,
            stack: vec![],
        }
    }

    fn frames_mut(&mut self) -> &mut Vec<ExecFrame> {
        &mut self.run.as_mut().unwrap().frames
    }

    fn get_frame_mut(&mut self, frame_id: u64) -> &mut ExecFrame {
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
    fn enter_node(&mut self, node_id: i64, function_name: &str) -> u64 {
        if self.run.is_none() {
            self.run = Some(TraceRun {
                frames: vec![],
                root: 0,
            });
        }

        let frame_id = self.next_id;
        self.next_id += 1;

        let frame = ExecFrame {
            frame_id,
            node_id,
            function_name: function_name.to_string(),
            args: vec![],
            outcome: None,
            start: Instant::now(),
            end: None,
            children: vec![],
        };

        let run = self.run.as_mut().unwrap();
        if run.root == 0 {
            run.root = frame_id;
        }
        run.frames.push(frame);

        self.stack.push(frame_id);
        frame_id
    }

    fn record_arg(&mut self, frame_id: u64, arg: ArgTrace) {
        self.get_frame_mut(frame_id).args.push(arg);
    }

    fn link_child(&mut self, parent_frame: u64, child_frame: u64, edge: EdgeKind) {
        self.get_frame_mut(parent_frame)
            .children
            .push((edge, child_frame));
    }

    fn exit_node(&mut self, frame_id: u64, outcome: Outcome) {
        let f = self.get_frame_mut(frame_id);
        f.outcome = Some(outcome);
        f.end = Some(Instant::now());

        // Pop in LIFO order
        let popped = self.stack.pop();
        debug_assert_eq!(popped, Some(frame_id));
    }
}
