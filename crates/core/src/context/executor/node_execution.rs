use super::*;

struct NodeHandler<'executor, 'deps> {
    executor: &'executor Executor<'deps>,
    node: NodeFunction,
}

impl<'executor, 'deps> NodeHandler<'executor, 'deps> {
    fn new(executor: &'executor Executor<'deps>, node: NodeFunction) -> Self {
        Self { executor, node }
    }

    fn execute(&self, ctx: &mut Context, tracer: &mut dyn ExecutionTracer) -> (Signal, u64) {
        if is_remote(&self.node) {
            self.execute_remote(ctx, tracer)
        } else {
            self.execute_local(ctx, tracer)
        }
    }

    fn execute_remote(&self, ctx: &mut Context, tracer: &mut dyn ExecutionTracer) -> (Signal, u64) {
        let remote = match self.executor.remote {
            Some(r) => r,
            None => {
                let err = RuntimeError::simple(
                    "RemoteRuntimeNotConfigured",
                    "Remote runtime not configured".to_string(),
                );
                return (Signal::Failure(err), 0);
            }
        };

        let frame_id = tracer.enter_node(
            self.node.database_id,
            self.node.runtime_function_id.as_str(),
        );

        let mut args = match self.executor.build_args(&self.node, ctx, tracer, frame_id) {
            Ok(a) => a,
            Err(e) => {
                ctx.insert_error(self.node.database_id, e.clone());
                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );
                return (Signal::Failure(e), frame_id);
            }
        };

        let values = match self
            .executor
            .resolve_remote_args(&mut args, ctx, tracer, frame_id)
        {
            Ok(v) => v,
            Err((sig, outcome)) => {
                tracer.exit_node(frame_id, outcome);
                return (sig, frame_id);
            }
        };

        let request = match self.executor.build_remote_request(&self.node, values) {
            Ok(r) => r,
            Err(e) => {
                ctx.insert_error(self.node.database_id, e.clone());
                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );
                return (Signal::Failure(e), frame_id);
            }
        };

        let remote_result =
            block_on(remote.execute_remote(self.node.definition_source.clone(), request));
        let signal = match remote_result {
            Ok(value) => Signal::Success(value),
            Err(err) => Signal::Failure(err),
        };

        let final_signal = self.commit_result(signal, ctx, tracer, frame_id);
        (final_signal, frame_id)
    }

    fn execute_local(&self, ctx: &mut Context, tracer: &mut dyn ExecutionTracer) -> (Signal, u64) {
        let entry = match self
            .executor
            .functions
            .get(self.node.runtime_function_id.as_str())
        {
            Some(e) => e,
            None => {
                let err = RuntimeError::simple(
                    "FunctionNotFound",
                    format!("Function {} not found", self.node.runtime_function_id),
                );
                return (Signal::Failure(err), 0);
            }
        };

        let frame_id = tracer.enter_node(
            self.node.database_id,
            self.node.runtime_function_id.as_str(),
        );

        // ---- Build args
        let mut args = match self.executor.build_args(&self.node, ctx, tracer, frame_id) {
            Ok(a) => a,
            Err(e) => {
                ctx.insert_error(self.node.database_id, e.clone());
                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );
                return (Signal::Failure(e), frame_id);
            }
        };

        // ---- Force eager args
        if let Err((sig, outcome)) = self
            .executor
            .force_eager_args(&self.node, entry, &mut args, ctx, tracer, frame_id)
        {
            tracer.exit_node(frame_id, outcome);
            return (sig, frame_id);
        }

        // ---- Invoke handler
        let result = self.invoke_handler(entry, &args, ctx, tracer, frame_id);

        // ---- Commit result
        let final_signal = self.commit_result(result, ctx, tracer, frame_id);
        (final_signal, frame_id)
    }

    /// Invoke a local handler with a closure for lazy execution.
    ///
    /// The closure will evaluate a thunk node and link its trace to the
    /// current execution frame.
    fn invoke_handler(
        &self,
        entry: &HandlerFunctionEntry,
        args: &[Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Signal {
        let executor = self.executor;
        let mut run = |node_id: i64, ctx: &mut Context| {
            tracer.mark_thunk_executed_by_node(frame_id, node_id);
            let label = ctx.pop_runtime_trace_label();
            let (sig, child_root) = executor.execute_call(node_id, ctx, tracer);
            tracer.link_child(frame_id, child_root, EdgeKind::RuntimeCall { label });
            match sig {
                // `return` only exits the child context and is a value for the caller.
                Signal::Return(v) => Signal::Success(v),
                other => other,
            }
        };

        (entry.handler)(args, ctx, &mut run)
    }

    /// Persist the final signal into the context and close the trace frame.
    fn commit_result(
        &self,
        result: Signal,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Signal {
        match result {
            Signal::Success(v) => {
                ctx.insert_success(self.node.database_id, v.clone());

                tracer.exit_node(
                    frame_id,
                    Outcome::Success {
                        value_preview: preview_value(&v),
                    },
                );

                Signal::Success(v)
            }

            Signal::Failure(e) => {
                ctx.insert_error(self.node.database_id, e.clone());

                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );

                Signal::Failure(e)
            }

            Signal::Return(v) => {
                tracer.exit_node(
                    frame_id,
                    Outcome::Return {
                        value_preview: preview_value(&v),
                    },
                );
                Signal::Return(v)
            }

            Signal::Respond(v) => {
                tracer.exit_node(
                    frame_id,
                    Outcome::Respond {
                        value_preview: preview_value(&v),
                    },
                );
                Signal::Respond(v)
            }

            Signal::Stop => {
                tracer.exit_node(frame_id, Outcome::Stop);
                Signal::Stop
            }
        }
    }
}

impl<'a> Executor<'a> {
    /// Execute a single node and return its signal and trace frame id.
    ///
    /// This handles:
    /// - Node lookup
    /// - Remote vs local dispatch
    /// - Argument building and eager evaluation
    /// - Handler invocation and result commit
    pub(super) fn execute_single_node(
        &self,
        node_id: i64,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> (Signal, u64) {
        ctx.set_current_node_id(node_id);
        let node = match self.nodes.get(&node_id) {
            Some(n) => n.clone(),
            None => {
                let err =
                    RuntimeError::simple("NodeNotFound", format!("Node {} not found", node_id));
                return (Signal::Failure(err), 0);
            }
        };

        NodeHandler::new(self, node).execute(ctx, tracer)
    }
}
