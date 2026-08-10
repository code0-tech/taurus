//! Registry of pending sub-flow node ranges.
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
//! parent remote call is outstanding -- the action may invoke the same
//! sub-flow UUID repeatedly. Only the parent call's own resolution (success
//! or failure) removes the entries it minted; see `EngineExecutor::execute_remote_node`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;

use crate::runtime::engine::model::CompiledFlow;

/// Everything needed to run one sub-flow node range standalone.
#[derive(Clone)]
pub struct PendingSubFlow {
    /// The compiled flow the referenced node range lives in. `CompiledFlow`
    /// is cheap to clone structurally, but it's wrapped in `Arc` at compile
    /// time (see `runtime/engine.rs`) so minting never deep-clones the node
    /// graph.
    pub flow: Arc<CompiledFlow>,
    /// Index of the sub-flow's entry node within `flow.nodes`.
    pub start_idx: usize,
    /// The *parent* flow execution's id -- reused as the `execution_identifier`
    /// for any remote call the sub-flow itself makes, and as the key for the
    /// idle-timeout activity marker (see `activity` below).
    pub parent_execution_id: String,
    /// Shared with the in-flight `NATSRemoteRuntime::execute_remote` call
    /// that minted this entry (and any sibling entries minted for the same
    /// remote call). Every successful lookup+run bumps this so that call's
    /// idle timeout resets instead of firing while the action is still
    /// actively driving sub-flow traffic.
    pub activity: Arc<Notify>,
}

/// Cheaply `Clone`-able handle to the shared pending sub-flow map. Safe to
/// hand a clone to both `EngineExecutor` (to mint) and the `sub_flow_execution.*`
/// NATS subscriber (to look up) since all state lives behind the shared `Arc`.
#[derive(Clone, Default)]
pub struct SubFlowRegistry {
    entries: Arc<Mutex<HashMap<String, PendingSubFlow>>>,
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
    pub fn mint(
        &self,
        flow: &Arc<CompiledFlow>,
        node_id: i64,
        parent_execution_id: &str,
        activity: Arc<Notify>,
    ) -> Option<String> {
        let start_idx = *flow.node_idx_by_id.get(&node_id)?;
        let id = uuid::Uuid::new_v4().to_string();
        let pending = PendingSubFlow {
            flow: Arc::clone(flow),
            start_idx,
            parent_execution_id: parent_execution_id.to_string(),
            activity,
        };
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .insert(id.clone(), pending);
        Some(id)
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

    /// Remove a single entry. Called once per minted id when the *parent*
    /// node's own remote call resolves (success or failure) -- not after
    /// each individual sub-flow invocation.
    pub fn remove(&self, id: &str) {
        self.entries
            .lock()
            .expect("sub flow registry mutex should not be poisoned")
            .remove(id);
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

        let id = registry
            .mint(&flow, 7, "parent-1", activity)
            .expect("node 7 exists in flow");

        let pending = registry.get(&id).expect("entry should exist after mint");
        assert_eq!(pending.start_idx, 0);
        assert_eq!(pending.parent_execution_id, "parent-1");

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

        assert!(registry.mint(&flow, 999, "parent-1", activity).is_none());
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
                        .mint(&flow, 7, &format!("parent-{i}"), activity)
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
}
