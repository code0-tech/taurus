//! Standalone workload for CPU profiling (flamegraph/samply), separate from
//! the Criterion harness so sampled stacks aren't diluted by Criterion's own
//! measurement loop overhead. Runs the pathological `chain_add` case
//! (Finding 1: O(n^2) trace snapshotting) many times in a tight loop.

use std::collections::HashMap;
use taurus_core::runtime::engine::ExecutionEngine;
use tucana::shared::{
    LiteralValue, NodeFunction, NodeParameter, NodeValue, ReferenceValue, node_value,
};

fn int_value(value: i64) -> tucana::shared::Value {
    taurus_core::value::value_from_i64(value)
}

fn literal_param(runtime_parameter_id: &str, value: tucana::shared::Value) -> NodeParameter {
    NodeParameter {
        database_id: 0,
        runtime_parameter_id: runtime_parameter_id.to_string(),
        value: Some(NodeValue {
            value: Some(node_value::Value::LiteralValue(LiteralValue {
                value: Some(value),
                references: Vec::new(),
            })),
        }),
        cast: None,
    }
}

fn reference_param(runtime_parameter_id: &str, node_id: i64) -> NodeParameter {
    NodeParameter {
        database_id: 0,
        runtime_parameter_id: runtime_parameter_id.to_string(),
        value: Some(NodeValue {
            value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                target: Some(tucana::shared::reference_value::Target::NodeId(node_id)),
                paths: Vec::new(),
            })),
        }),
        cast: None,
    }
}

fn node(
    database_id: i64,
    parameters: Vec<NodeParameter>,
    next_node_id: Option<i64>,
) -> NodeFunction {
    NodeFunction {
        database_id: Some(database_id),
        runtime_function_id: "std::number::add".to_string(),
        parameters,
        next_node_id,
        definition_source: Some("taurus".to_string()),
    }
}

fn build_chain_flow(n: i64) -> (i64, Vec<NodeFunction>) {
    let mut nodes = Vec::with_capacity(n as usize);
    nodes.push(node(
        1,
        vec![
            literal_param("a", int_value(0)),
            literal_param("b", int_value(1)),
        ],
        if n > 1 { Some(2) } else { None },
    ));
    for i in 2..=n {
        nodes.push(node(
            i,
            vec![
                reference_param("a", i - 1),
                literal_param("b", int_value(1)),
            ],
            if i < n { Some(i + 1) } else { None },
        ));
    }
    (1, nodes)
}

fn main() {
    let with_trace = std::env::var("PROFILE_TRACE").is_ok();
    let iterations: u32 = std::env::var("PROFILE_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let engine = ExecutionEngine::new();
    let (start, nodes) = build_chain_flow(200);

    let mut total = HashMap::new();
    for i in 0..iterations {
        let (signal, _reason) =
            engine.execute_graph("bench", start, nodes.clone(), None, None, with_trace);
        total.insert(i, signal.exit_reason());
    }
    // Prevent the compiler from optimizing the loop away.
    eprintln!(
        "completed {} iterations, last exit_reason={:?}",
        iterations,
        total.get(&(iterations.saturating_sub(1)))
    );
}
