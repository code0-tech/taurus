use crate::context::argument::{Argument, ParameterNode};
use crate::context::context::{Context, ContextResult};
use crate::context::registry::{FunctionStore, HandlerFunctionEntry};
use crate::context::signal::Signal;
use crate::debug::trace::{ArgKind, ArgTrace, EdgeKind, Outcome, ReferenceKind};
use crate::debug::tracer::{ExecutionTracer, Tracer};
use crate::error::RuntimeError;

use std::collections::HashMap;
use tucana::shared::NodeFunction;

pub struct Executor<'a> {
    functions: &'a FunctionStore,
    nodes: HashMap<i64, NodeFunction>,
}

impl<'a> Executor<'a> {
    pub fn new(functions: &'a FunctionStore, nodes: HashMap<i64, NodeFunction>) -> Self {
        Self { functions, nodes }
    }

    /// This is now the ONLY execution entry point.
    pub fn execute(&self, start_node_id: i64, ctx: &mut Context) -> Signal {
        let mut tracer = Tracer::new();

        let (signal, _root_frame) = self.execute_call(start_node_id, ctx, &mut tracer);

        if let Some(run) = tracer.take_run() {
            println!("{}", crate::debug::render::render_trace(&run));
        }

        signal
    }

    // Main execution loop
    fn execute_call(
        &self,
        start_node_id: i64,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> (Signal, u64) {
        let mut current = start_node_id;

        let mut call_root_frame: Option<u64> = None;
        let mut previous_frame: Option<u64> = None;

        loop {
            let (signal, frame_id) = self.execute_single_node(current, ctx, tracer);

            if call_root_frame.is_none() {
                call_root_frame = Some(frame_id);
            }

            // Link linear NEXT chain
            if let Some(prev) = previous_frame {
                tracer.link_child(prev, frame_id, EdgeKind::Next);
            }
            previous_frame = Some(frame_id);

            match signal {
                Signal::Success(_) => {
                    let node = self.nodes.get(&current).unwrap();

                    if let Some(next) = node.next_node_id {
                        current = next;
                        continue;
                    }

                    return (signal, call_root_frame.unwrap());
                }

                _ => return (signal, call_root_frame.unwrap()),
            }
        }
    }

    // executes a single node
    fn execute_single_node(
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

        let entry = match self.functions.get(node.runtime_function_id.as_str()) {
            Some(e) => e,
            None => {
                let err = RuntimeError::simple(
                    "FunctionNotFound",
                    format!("Function {} not found", node.runtime_function_id),
                );
                return (Signal::Failure(err), 0);
            }
        };

        let frame_id = tracer.enter_node(node.database_id, node.runtime_function_id.as_str());

        // ---- Build args
        let mut args = match self.build_args(&node, ctx, tracer, frame_id) {
            Ok(a) => a,
            Err(e) => {
                ctx.insert_error(node.database_id, e.clone());
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
        if let Err((sig, outcome)) =
            self.force_eager_args(&node, entry, &mut args, ctx, tracer, frame_id)
        {
            tracer.exit_node(frame_id, outcome);
            return (sig, frame_id);
        }

        // ---- Invoke handler
        let result = self.invoke_handler(entry, &args, ctx, tracer);

        // ---- Commit result
        let final_signal = self.commit_result(&node, result, ctx, tracer, frame_id);

        (final_signal, frame_id)
    }

    fn build_args(
        &self,
        node: &NodeFunction,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Result<Vec<Argument>, RuntimeError> {
        let mut args = Vec::with_capacity(node.parameters.len());

        for (i, param) in node.parameters.iter().enumerate() {
            let node_value = param.value.as_ref().ok_or_else(|| {
                RuntimeError::simple_str("NodeValueNotFound", "Missing param value")
            })?;

            let value = node_value.value.as_ref().ok_or_else(|| {
                RuntimeError::simple_str("NodeValueNotFound", "Missing inner value")
            })?;

            match value {
                tucana::shared::node_value::Value::LiteralValue(v) => {
                    tracer.record_arg(
                        frame_id,
                        ArgTrace {
                            index: i,
                            kind: ArgKind::Literal,
                            preview: format!("{:?}", v),
                        },
                    );
                    args.push(Argument::Eval(v.clone()));
                }

                tucana::shared::node_value::Value::ReferenceValue(r) => match ctx.get(r.clone()) {
                    ContextResult::Success(v) => {
                        let reference = match r.target {
                            Some(ref_value) => match ref_value {
                                tucana::shared::reference_value::Target::FlowInput(_) => {
                                    ReferenceKind::FlowInput
                                }
                                tucana::shared::reference_value::Target::NodeId(id) => {
                                    ReferenceKind::Result { node_id: id }
                                }
                                tucana::shared::reference_value::Target::InputType(input_type) => {
                                    ReferenceKind::InputType {
                                        node_id: input_type.node_id,
                                        input_index: input_type.input_index,
                                        parameter_index: input_type.parameter_index,
                                    }
                                }
                            },
                            None => ReferenceKind::Empty,
                        };

                        tracer.record_arg(
                            frame_id,
                            ArgTrace {
                                index: i,
                                kind: ArgKind::Reference {
                                    reference: reference,
                                    hit: true,
                                },
                                preview: format!("ctx.get({:?}) -> {:?}", r, v),
                            },
                        );
                        args.push(Argument::Eval(v));
                    }
                    ContextResult::Error(e) => return Err(e),
                    ContextResult::NotFound => {
                        return Err(RuntimeError::simple_str(
                            "ReferenceValueNotFound",
                            "Referenced node not executed",
                        ));
                    }
                },

                tucana::shared::node_value::Value::NodeFunctionId(id) => {
                    tracer.record_arg(
                        frame_id,
                        ArgTrace {
                            index: i,
                            kind: ArgKind::Thunk {
                                node_id: *id,
                                eager: false,
                                executed: false,
                            },
                            preview: format!("thunk({})", id),
                        },
                    );
                    args.push(Argument::Thunk(*id));
                }
            }
        }

        Ok(args)
    }

    fn force_eager_args(
        &self,
        _node: &NodeFunction,
        entry: &HandlerFunctionEntry,
        args: &mut [Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        parent_frame: u64,
    ) -> Result<(), (Signal, Outcome)> {
        for (i, arg) in args.iter_mut().enumerate() {
            let mode = entry
                .param_modes
                .get(i)
                .copied()
                .unwrap_or(ParameterNode::Eager);

            if matches!(mode, ParameterNode::Eager) {
                if let Argument::Thunk(id) = *arg {
                    let (child_sig, child_root) = self.execute_call(id, ctx, tracer);

                    tracer.link_child(
                        parent_frame,
                        child_root,
                        EdgeKind::EagerCall { arg_index: i },
                    );

                    match child_sig {
                        Signal::Success(v) => {
                            *arg = Argument::Eval(v);
                        }
                        s => {
                            return Err((
                                s,
                                Outcome::Failure {
                                    error_preview: "Eager child failed".into(),
                                },
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn invoke_handler(
        &self,
        entry: &HandlerFunctionEntry,
        args: &[Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> Signal {
        let mut run = |node_id: i64, ctx: &mut Context| {
            let (sig, _) = self.execute_call(node_id, ctx, tracer);
            sig
        };

        (entry.handler)(args, ctx, &mut run)
    }

    fn commit_result(
        &self,
        node: &NodeFunction,
        result: Signal,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Signal {
        match result {
            Signal::Success(v) => {
                ctx.insert_success(node.database_id, v.clone());

                tracer.exit_node(
                    frame_id,
                    Outcome::Success {
                        value_preview: format!("{:#?}", v),
                    },
                );

                Signal::Success(v)
            }

            Signal::Failure(e) => {
                ctx.insert_error(node.database_id, e.clone());

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
                        value_preview: format!("{:#?}", v),
                    },
                );
                Signal::Return(v)
            }

            Signal::Respond(v) => {
                tracer.exit_node(
                    frame_id,
                    Outcome::Respond {
                        value_preview: format!("{:#?}", v),
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
