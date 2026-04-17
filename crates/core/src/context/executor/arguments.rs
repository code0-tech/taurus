use super::*;

impl<'a> Executor<'a> {
    /// Build arguments for a node from literals, references, or thunks.
    ///
    /// Arguments are recorded to the tracer for debugging and inspection.
    /// Thunks are represented by the referenced node id.
    pub(super) fn build_args(
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
                            preview: preview_value(v),
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
                                    reference,
                                    hit: true,
                                },
                                preview: format!(
                                    "ctx.get({}) -> {}",
                                    preview_reference(r),
                                    preview_value(&v)
                                ),
                            },
                        );
                        args.push(Argument::Eval(v));
                    }
                    ContextResult::Error(e) => return Err(e),
                    ContextResult::NotFound => {
                        return Err(RuntimeError::simple_str(
                            "ReferenceValueNotFound",
                            "Reference not found in context",
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

    /// Eagerly execute any argument marked as `Eager`.
    ///
    /// Lazy arguments are preserved as thunks until needed by a handler.
    /// If an eager child fails, the failure bubbles up immediately.
    pub(super) fn force_eager_args(
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

            if matches!(mode, ParameterNode::Eager)
                && let Argument::Thunk(id) = *arg
            {
                tracer.mark_thunk(parent_frame, i, true, true);
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
                    Signal::Return(v) => {
                        // `return` only exits the child context and yields a value upward.
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

        Ok(())
    }

    /// Resolve all arguments for a remote call.
    ///
    /// Remote execution requires concrete values, so any thunks are executed
    /// eagerly and replaced with their resulting `Value`.
    pub(super) fn resolve_remote_args(
        &self,
        args: &mut [Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        parent_frame: u64,
    ) -> Result<Vec<Value>, (Signal, Outcome)> {
        let mut values = Vec::with_capacity(args.len());

        for (i, arg) in args.iter_mut().enumerate() {
            match arg {
                Argument::Eval(v) => values.push(v.clone()),
                Argument::Thunk(id) => {
                    tracer.mark_thunk(parent_frame, i, true, true);
                    let (child_sig, child_root) = self.execute_call(*id, ctx, tracer);
                    if child_root != 0 {
                        tracer.link_child(
                            parent_frame,
                            child_root,
                            EdgeKind::EagerCall { arg_index: i },
                        );
                    }

                    match child_sig {
                        Signal::Success(v) => {
                            *arg = Argument::Eval(v.clone());
                            values.push(v);
                        }
                        Signal::Return(v) => {
                            // `return` only exits the child context and yields a value upward.
                            *arg = Argument::Eval(v.clone());
                            values.push(v);
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

        Ok(values)
    }

    /// Build a remote execution request from a node and its resolved values.
    ///
    /// Values are mapped to their parameter ids for the remote runtime.
    pub(super) fn build_remote_request(
        &self,
        node: &NodeFunction,
        values: Vec<Value>,
    ) -> Result<ExecutionRequest, RuntimeError> {
        if node.parameters.len() != values.len() {
            return Err(RuntimeError::simple_str(
                "RemoteParameterMismatch",
                "Remote parameter count mismatch",
            ));
        }

        let mut fields = HashMap::new();
        for (param, value) in node.parameters.iter().zip(values.into_iter()) {
            fields.insert(param.runtime_parameter_id.clone(), value);
        }
        let id = Uuid::new_v4();
        Ok(ExecutionRequest {
            execution_identifier: id.to_string(),
            function_identifier: node.runtime_function_id.clone(),
            parameters: Some(Struct { fields }),
            project_id: 0,
        })
    }
}
