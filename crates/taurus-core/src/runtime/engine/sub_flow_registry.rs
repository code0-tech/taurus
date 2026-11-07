//! Registry of pending sub-flow node ranges -- and of sub flow functions a
//! remote runtime returned and Taurus handed on to another remote node.
//!
//! `execute_compiled` holds `EngineExecutor`/`&mut ValueStore` borrowed for
//! the entire span a remote node call is outstanding (parked on one
//! `.await` in `execute_remote_node`) -- a separate NATS subscriber
//! (`sub_flow_execution.*` in `taurus/src/app/worker.rs`) can't reach back
//! into that borrowed state to run a sub-flow node range on demand. So
//! registry entries are self-contained: everything needed to run the
//! sub-flow's node range standalone, captured at mint time.
//!
//! Entries are looked up (never removed) any number of times while the
//! flow execution that minted them is still running -- the action may invoke
//! the same sub-flow UUID repeatedly, and a returned sub flow function may be
//! handed on and invoked by a *later* node, long after the call that minted
//! its references resolved. So an entry's lifetime is tied to the whole flow
//! execution, not to the single remote call that minted it: the top-level run
//! sweeps every entry it owns when it finishes (`remove_for_execution`, called
//! from `ExecutionEngine::execute_graph_with_project_id_report_async`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;
use tucana::shared::NodeExecutionResult;

use crate::handler::argument::FunctionThunk;
use crate::runtime::engine::model::CompiledFlow;

/// What a pending sub flow runs when the action invokes it.
#[derive(Clone)]
pub enum PendingSubFlowTarget {
    /// A node range of `PendingSubFlow::flow`, starting at this index of
    /// `flow.nodes`.
    NodeRange { start_idx: usize },
    /// A sub flow function a remote runtime returned as a node result, run
    /// with the inputs the action passes in.
    Function(FunctionThunk),
}

/// Everything needed to run one sub-flow standalone.
#[derive(Clone)]
pub struct PendingSubFlow {
    /// The compiled flow the sub-flow was minted from (the referenced node
    /// range lives in it). `CompiledFlow` is cheap to clone structurally,
    /// but it's wrapped in `Arc` at compile time (see `runtime/engine.rs`)
    /// so minting never deep-clones the node graph.
    pub flow: Arc<CompiledFlow>,
    pub target: PendingSubFlowTarget,
    /// Execution id of the remote call that minted this entry. Informational
    /// (logging/correlation) now that cleanup is keyed on `flow_execution_id`;
    /// the sub-flow's own remote calls run under a fresh id (see
    /// `ExecutionEngine::execute_sub_flow`).
    pub parent_execution_id: String,
    /// Id of the *top-level* flow execution that owns this entry. All entries
    /// minted anywhere under one flow run -- including by sub-flow runs nested
    /// inside it -- carry the same value, so the top-level run can sweep them
    /// all at once when it finishes. This is what lets a returned sub flow
    /// function outlive the call that minted its references (code0-tech/taurus#346).
    pub flow_execution_id: String,
    /// Id of the node whose parameter this sub-flow was minted from (e.g.
    /// the `for_each` node), and that parameter's positional index (e.g.
    /// `consumer`). Together with each call's positional slot these form the
    /// `InputType{node_id, parameter_index, input_index}` key the sub-flow's
    /// node range references for its action-supplied arguments -- the same
    /// addressing a local consumer callback gets via
    /// `ValueStore::insert_input_type` (see `functions/array.rs::run_with_unary_input`).
    /// `execute_sub_flow` seeds the store with these before running the
    /// node range standalone.
    pub caller_node_id: i64,
    pub caller_parameter_index: i64,
    /// The flow execution's shared idle-timeout activity (see
    /// `SubFlowRegistry::activity_for`) -- the same `Notify` every remote call
    /// of this `flow_execution_id` waits on, not just the one that minted this
    /// entry. Every successful lookup+run bumps it so that whichever remote
    /// call is currently driving sub-flow traffic has its idle timeout reset,
    /// even though it may be a later call than the one that minted this
    /// reference (which the old minting-call-scoped activity could not renew).
    pub activity: Arc<Notify>,
}

/// Cheaply `Clone`-able handle to the shared pending sub-flow map. Safe to
/// hand a clone to both `EngineExecutor` (to mint) and the `sub_flow_execution.*`
/// NATS subscriber (to look up) since all state lives behind the shared `Arc`.
#[derive(Clone, Default)]
pub struct SubFlowRegistry {
    entries: Arc<Mutex<HashMap<String, PendingSubFlow>>>,
    /// One idle-timeout activity `Notify` per *flow execution*, shared by
    /// every remote call that flow makes and bumped on every sub-flow lookup
    /// for that flow (see `activity_for`). Keyed by `flow_execution_id` and
    /// swept by `remove_for_execution`. A minted reference now lives for the
    /// whole flow execution and may be invoked by a later call than the one
    /// that minted it, so the renewable window has to follow the flow rather
    /// than the single minting call.
    activities: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
    /// Node execution results collected from every sub-flow run of a flow
    /// execution (keyed by `flow_execution_id`). A remote sub-flow invocation
    /// runs standalone in `ExecutionEngine::execute_sub_flow` with its own
    /// value store, so its results never reach the parent run's store the way
    /// a locally-run consumer sub flow's do. Each run records its results here
    /// (`record_sub_flow_results`) and the top-level run drains them
    /// (`take_sub_flow_results`) into its own `node_execution_results`, so the
    /// execution result lists every executed function with its params --
    /// whether it ran in the top-level flow, a local consumer, or a remote
    /// sub-flow invocation.
    sub_flow_results: Arc<Mutex<HashMap<String, Vec<NodeExecutionResult>>>>,
}

impl SubFlowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a fresh execution id for `node_id` inside `flow` and register it.
    ///
    /// Returns `None` if `node_id` isn't a real node in `flow` -- defensive:
    /// the compiler is expected to reject a dangling sub-flow reference
    /// before this is ever reached, but minting must not panic on a bad id.
    #[allow(clippy::too_many_arguments)]
    pub fn mint(
        &self,
        flow: &Arc<CompiledFlow>,
        node_id: i64,
        parent_execution_id: &str,
        flow_execution_id: &str,
        activity: Arc<Notify>,
        caller_node_id: i64,
        caller_parameter_index: i64,
    ) -> Option<String> {
        let start_idx = *flow.node_idx_by_id.get(&node_id)?;
        Some(self.insert(PendingSubFlow {
            flow: Arc::clone(flow),
            target: PendingSubFlowTarget::NodeRange { start_idx },
            parent_execution_id: parent_execution_id.to_string(),
            flow_execution_id: flow_execution_id.to_string(),
            activity,
            caller_node_id,
            caller_parameter_index,
        }))
    }

    /// Mint a fresh execution id for a returned sub flow function handed on
    /// to a remote node, and register it.
    #[allow(clippy::too_many_arguments)]
    pub fn mint_function(
        &self,
        flow: &Arc<CompiledFlow>,
        function: FunctionThunk,
        parent_execution_id: &str,
        flow_execution_id: &str,
        activity: Arc<Notify>,
        caller_node_id: i64,
        caller_parameter_index: i64,
    ) -> String {
        self.insert(PendingSubFlow {
            flow: Arc::clone(flow),
            target: PendingSubFlowTarget::Function(function),
            parent_execution_id: parent_execution_id.to_string(),
            flow_execution_id: flow_execution_id.to_string(),
            activity,
            caller_node_id,
            caller_parameter_index,
        })
    }

    /// The shared idle-timeout activity for `flow_execution_id`, created on
    /// first use and reused by every subsequent remote call and sub-flow
    /// lookup of that flow execution. Every remote call (whether or not it
    /// mints a reference) waits on this, and `ExecutionEngine::execute_sub_flow`
    /// bumps it on every lookup+run, so any remote call of a flow that is
    /// still making sub-flow progress has its idle deadline renewed -- not
    /// only the call that happened to mint the reference being invoked. Swept
    /// by `remove_for_execution`.
    pub fn activity_for(&self, flow_execution_id: &str) -> Arc<Notify> {
        self.activities
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .entry(flow_execution_id.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    /// Record the node execution results of one sub-flow run so the owning
    /// top-level flow execution can fold them into its own result. Appends, so
    /// a sub flow invoked many times (and nested sub flows, which carry the
    /// same `flow_execution_id`) all accumulate under one key in run order.
    pub fn record_sub_flow_results(
        &self,
        flow_execution_id: &str,
        results: Vec<NodeExecutionResult>,
    ) {
        if results.is_empty() {
            return;
        }
        self.sub_flow_results
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .entry(flow_execution_id.to_string())
            .or_default()
            .extend(results);
    }

    /// Remove and return every sub-flow node execution result collected for a
    /// flow execution. Called once by the top-level run when it finishes, to
    /// merge them into its `node_execution_results` (see
    /// `ExecutionEngine::execute_graph_with_project_id_report_async`).
    pub fn take_sub_flow_results(&self, flow_execution_id: &str) -> Vec<NodeExecutionResult> {
        self.sub_flow_results
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .remove(flow_execution_id)
            .unwrap_or_default()
    }

    fn insert(&self, pending: PendingSubFlow) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .insert(id.clone(), pending);
        id
    }

    /// Look up a pending sub-flow without removing it -- the same id can be
    /// looked up many times while the parent call is outstanding.
    pub fn get(&self, id: &str) -> Option<PendingSubFlow> {
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    /// Remove a single entry. Used on paths where a minted reference is known
    /// to be orphaned before it could ever escape -- e.g. the remote call it
    /// was minted for failed to even be built, so no action ever received it.
    pub fn remove(&self, id: &str) {
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .remove(id);
    }

    /// Remove every entry owned by a top-level flow execution. Called once,
    /// when that execution finishes, so all references it minted -- including
    /// those a returned sub flow function carried across nodes -- stay alive
    /// for exactly as long as the flow can still invoke them.
    pub fn remove_for_execution(&self, flow_execution_id: &str) {
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .retain(|_, pending| pending.flow_execution_id != flow_execution_id);
        self.activities
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .remove(flow_execution_id);
        // Defensive: the top-level run drains these via `take_sub_flow_results`
        // before sweeping, but clear any straggler (e.g. a late record that
        // lost the race) so a finished flow execution leaves nothing behind.
        self.sub_flow_results
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .remove(flow_execution_id);
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::engine::model::CompiledFlow;
    use std::collections::HashMap as StdHashMap;

    fn flow_with_node(node_id: i64) -> Arc<CompiledFlow> {
        Arc::new(CompiledFlow {
            project_id: 1,
            start_idx: 0,
            nodes: vec![crate::runtime::engine::model::CompiledNode {
                id: node_id,
                handler_id: "test::handler".to_string(),
                execution_target: crate::runtime::engine::model::NodeExecutionTarget::Local,
                next_idx: None,
                parameters: Vec::new(),
            }],
            node_idx_by_id: StdHashMap::from([(node_id, 0usize)]),
        })
    }

    #[test]
    fn mint_get_remove_round_trip() {
        let registry = SubFlowRegistry::new();
        let flow = flow_with_node(7);
        let activity = Arc::new(Notify::new());

        // caller_node_id and caller_parameter_index are deliberately distinct
        // (3, 5) so a field mix-up in `mint`/`PendingSubFlow` would be
        // caught by the assertions below instead of accidentally matching.
        let id = registry
            .mint(&flow, 7, "parent-1", "flow-1", activity, 3, 5)
            .expect("node 7 exists in flow");

        let pending = registry.get(&id).expect("entry should exist after mint");
        assert!(matches!(
            pending.target,
            PendingSubFlowTarget::NodeRange { start_idx: 0 }
        ));
        assert_eq!(pending.parent_execution_id, "parent-1");
        assert_eq!(pending.flow_execution_id, "flow-1");
        assert_eq!(pending.caller_node_id, 3);
        assert_eq!(pending.caller_parameter_index, 5);

        // Looking up again does not remove the entry.
        assert!(registry.get(&id).is_some());

        registry.remove(&id);
        assert!(registry.get(&id).is_none());
    }

    #[test]
    fn mint_returns_none_for_unknown_node_id() {
        let registry = SubFlowRegistry::new();
        let flow = flow_with_node(7);
        let activity = Arc::new(Notify::new());

        assert!(
            registry
                .mint(&flow, 999, "parent-1", "flow-1", activity, 1, 1)
                .is_none()
        );
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn concurrent_mints_from_simulated_parallel_parents_all_survive() {
        let registry = SubFlowRegistry::new();
        let flow = flow_with_node(7);

        let handles: Vec<_> = (0..32)
            .map(|i| {
                let registry = registry.clone();
                let flow = Arc::clone(&flow);
                std::thread::spawn(move || {
                    let activity = Arc::new(Notify::new());
                    registry
                        .mint(&flow, 7, &format!("parent-{i}"), "flow-1", activity, 1, 1)
                        .expect("node 7 exists in flow")
                })
            })
            .collect();

        let ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(ids.len(), 32);
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 32, "every mint should produce a unique id");
        assert_eq!(registry.len(), 32);

        for id in &ids {
            assert!(registry.get(id).is_some());
        }
    }

    #[test]
    fn activity_is_shared_per_flow_execution_and_swept_with_it() {
        let registry = SubFlowRegistry::new();

        // Every remote call of one flow execution must wait on the *same*
        // `Notify`, so a sub-flow invocation renews whichever call is driving
        // it -- not just the one that minted the reference.
        let a1 = registry.activity_for("flow-a");
        let a2 = registry.activity_for("flow-a");
        let b = registry.activity_for("flow-b");
        assert!(Arc::ptr_eq(&a1, &a2), "same flow execution shares one Notify");
        assert!(
            !Arc::ptr_eq(&a1, &b),
            "different flow executions get distinct Notifys"
        );

        // Sweeping a flow drops its activity (no leak), and an unrelated flow
        // is untouched.
        registry.remove_for_execution("flow-a");
        let a3 = registry.activity_for("flow-a");
        assert!(
            !Arc::ptr_eq(&a1, &a3),
            "a swept flow's next activity is a fresh Notify"
        );
        assert!(
            Arc::ptr_eq(&b, &registry.activity_for("flow-b")),
            "sweeping one flow leaves another's activity intact"
        );
    }

    #[test]
    fn mint_function_registers_the_returned_function() {
        let registry = SubFlowRegistry::new();
        let flow = flow_with_node(7);
        let activity = Arc::new(Notify::new());
        let function = FunctionThunk {
            identifier: "remote::handler".to_string(),
            execution_target: crate::runtime::engine::model::NodeExecutionTarget::Remote {
                service: "example".to_string(),
            },
            parameter_index: 5,
            settings: Vec::new(),
            input_schema: None,
            output_schema: None,
            name: None,
            description: None,
            sub_flow_function: None,
        };

        let id = registry.mint_function(&flow, function, "parent-1", "flow-1", activity, 3, 5);

        let pending = registry.get(&id).expect("entry should exist after mint");
        match pending.target {
            PendingSubFlowTarget::Function(function) => {
                assert_eq!(function.identifier, "remote::handler");
            }
            PendingSubFlowTarget::NodeRange { .. } => panic!("expected a function target"),
        }
        assert_eq!(pending.caller_node_id, 3);
        assert_eq!(pending.caller_parameter_index, 5);

        registry.remove(&id);
        assert!(registry.get(&id).is_none());
    }

    #[test]
    fn remove_for_execution_only_sweeps_its_own_flow() {
        let registry = SubFlowRegistry::new();
        let flow = flow_with_node(7);

        // Two entries under flow A (the one we sweep) and one under flow B.
        let a1 = registry
            .mint(&flow, 7, "parent-a", "flow-a", Arc::new(Notify::new()), 1, 1)
            .expect("node 7 exists in flow");
        let a2 = registry.mint_function(
            &flow,
            FunctionThunk {
                identifier: "remote::handler".to_string(),
                execution_target:
                    crate::runtime::engine::model::NodeExecutionTarget::Remote {
                        service: "example".to_string(),
                    },
                parameter_index: 0,
                settings: Vec::new(),
                input_schema: None,
                output_schema: None,
                name: None,
                description: None,
                sub_flow_function: None,
            },
            "parent-a",
            "flow-a",
            Arc::new(Notify::new()),
            1,
            1,
        );
        let b1 = registry
            .mint(&flow, 7, "parent-b", "flow-b", Arc::new(Notify::new()), 1, 1)
            .expect("node 7 exists in flow");

        registry.remove_for_execution("flow-a");

        assert!(registry.get(&a1).is_none(), "flow-a node range swept");
        assert!(registry.get(&a2).is_none(), "flow-a function swept");
        assert!(registry.get(&b1).is_some(), "flow-b entry untouched");
    }
}
