use super::*;

#[derive(Debug)]
struct CallState {
    current_node_id: i64,
    call_root_frame: Option<u64>,
    previous_frame: Option<u64>,
}

impl CallState {
    fn new(start_node_id: i64) -> Self {
        Self {
            current_node_id: start_node_id,
            call_root_frame: None,
            previous_frame: None,
        }
    }

    fn set_root_if_missing(&mut self, frame_id: u64) {
        if self.call_root_frame.is_none() {
            self.call_root_frame = Some(frame_id);
        }
    }

    fn link_next_edge(&mut self, frame_id: u64, tracer: &mut dyn ExecutionTracer) {
        if let Some(prev) = self.previous_frame {
            tracer.link_child(prev, frame_id, EdgeKind::Next);
        }
        self.previous_frame = Some(frame_id);
    }

    fn advance(&mut self, next_node_id: Option<i64>) -> bool {
        if let Some(next) = next_node_id {
            self.current_node_id = next;
            true
        } else {
            false
        }
    }

    fn root_frame(&self) -> u64 {
        self.call_root_frame
            .expect("call root frame must be set after first node execution")
    }
}

impl<'a> Executor<'a> {
    /// Main execution loop.
    ///
    /// Executes nodes one-by-one along the `next_node_id` chain until a
    /// non-success signal is produced or the chain ends.
    pub(super) fn execute_call(
        &self,
        start_node_id: i64,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> (Signal, u64) {
        let mut state = CallState::new(start_node_id);

        loop {
            let (signal, frame_id) = self.execute_single_node(state.current_node_id, ctx, tracer);
            state.set_root_if_missing(frame_id);
            state.link_next_edge(frame_id, tracer);

            match signal {
                Signal::Success(_) => {
                    let node = self
                        .nodes
                        .get(&state.current_node_id)
                        .expect("current node must exist while executing success path");

                    if state.advance(node.next_node_id) {
                        continue;
                    }

                    return (signal, state.root_frame());
                }

                Signal::Respond(v) => {
                    let node = self
                        .nodes
                        .get(&state.current_node_id)
                        .expect("current node must exist while executing respond path");

                    if let Some(emit) = self.respond_emitter {
                        emit(v.clone());
                    }

                    // Respond behaves like success for flow continuation and references.
                    ctx.insert_success(node.database_id, v.clone());

                    if state.advance(node.next_node_id) {
                        continue;
                    }

                    return (Signal::Success(v), state.root_frame());
                }

                _ => return (signal, state.root_frame()),
            }
        }
    }
}
