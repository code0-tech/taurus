//! Flow-level benchmarks for `ExecutionEngine`.
//!
//! `chain_add` measures per-node overhead across a linear chain (each node
//! reads the previous node's result), which is where `ValueStore` result
//! storage and reference resolution cost accumulates. `array_map` measures
//! per-iteration callback overhead inside a single `map` node, which is
//! where unconditional trace-label formatting accumulates independently of
//! node count. Both are run with `with_trace` on and off to isolate the
//! cost of Trace V2 collection.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use taurus_core::runtime::engine::ExecutionEngine;
use tucana::shared::{
    ListValue, NodeFunction, NodeParameter, NodeValue, ReferenceValue, Struct, SubFlow, Value,
    node_value, reference_value, reference_value::Target, sub_flow::ExecutionReference,
    value::Kind,
};

fn int_value(value: i64) -> Value {
    taurus_core::value::value_from_i64(value)
}

fn literal_param(runtime_parameter_id: &str, value: Value) -> NodeParameter {
    NodeParameter {
        database_id: 0,
        runtime_parameter_id: runtime_parameter_id.to_string(),
        value: Some(NodeValue {
            value: Some(node_value::Value::LiteralValue(value)),
        }),
        cast: None,
    }
}

fn reference_param(runtime_parameter_id: &str, target: Target) -> NodeParameter {
    NodeParameter {
        database_id: 0,
        runtime_parameter_id: runtime_parameter_id.to_string(),
        value: Some(NodeValue {
            value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                target: Some(target),
                paths: Vec::new(),
            })),
        }),
        cast: None,
    }
}

fn thunk_param(runtime_parameter_id: &str, starting_node_id: i64) -> NodeParameter {
    NodeParameter {
        database_id: 0,
        runtime_parameter_id: runtime_parameter_id.to_string(),
        value: Some(NodeValue {
            value: Some(node_value::Value::SubFlow(SubFlow {
                input_schema: None,
                output_schema: None,
                signature: String::new(),
                settings: Vec::new(),
                execution_reference: Some(ExecutionReference::StartingNodeId(starting_node_id)),
            })),
        }),
        cast: None,
    }
}

fn node(
    database_id: i64,
    runtime_function_id: &str,
    parameters: Vec<NodeParameter>,
    next_node_id: Option<i64>,
) -> NodeFunction {
    NodeFunction {
        database_id: Some(database_id),
        runtime_function_id: runtime_function_id.to_string(),
        parameters,
        next_node_id,
        definition_source: Some("taurus".to_string()),
    }
}

/// A chain of `n` "std::number::add" nodes; node `i` adds a constant to
/// node `i - 1`'s result. Exercises per-node ValueStore insert/lookup cost.
fn build_chain_flow(n: i64) -> (i64, Vec<NodeFunction>) {
    let mut nodes = Vec::with_capacity(n as usize);
    nodes.push(node(
        1,
        "std::number::add",
        vec![
            literal_param("a", int_value(0)),
            literal_param("b", int_value(1)),
        ],
        if n > 1 { Some(2) } else { None },
    ));
    for i in 2..=n {
        nodes.push(node(
            i,
            "std::number::add",
            vec![
                reference_param("a", Target::NodeId(i - 1)),
                literal_param("b", int_value(1)),
            ],
            if i < n { Some(i + 1) } else { None },
        ));
    }
    (1, nodes)
}

/// A single "std::list::map" node over an array of `n` items, with a
/// callback (node 2) that adds a constant to each item. Exercises
/// per-iteration callback overhead independent of node count.
fn build_map_flow(n: usize) -> (i64, Vec<NodeFunction>) {
    let array = Value {
        kind: Some(Kind::ListValue(ListValue {
            values: (0..n as i64).map(int_value).collect(),
        })),
    };

    let map_node = node(
        1,
        "std::list::map",
        vec![literal_param("array", array), thunk_param("transform", 2)],
        None,
    );
    let callback_node = node(
        2,
        "std::number::add",
        vec![
            reference_param(
                "a",
                Target::InputType(tucana::shared::InputType {
                    node_id: 1,
                    parameter_index: 1,
                    input_index: 0,
                }),
            ),
            literal_param("b", int_value(1)),
        ],
        None,
    );
    (1, vec![map_node, callback_node])
}

fn bench_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_add");
    for &size in &[10_i64, 50, 200] {
        for &with_trace in &[false, true] {
            let (start, nodes) = build_chain_flow(size);
            group.bench_with_input(
                BenchmarkId::new(if with_trace { "trace_on" } else { "trace_off" }, size),
                &size,
                |b, _| {
                    let engine = ExecutionEngine::new();
                    b.iter(|| engine.execute_graph("bench", start, nodes.clone(), None, None, with_trace));
                },
            );
        }
    }
    group.finish();
}

fn bench_array_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_map");
    for &size in &[10_usize, 100, 1_000] {
        for &with_trace in &[false, true] {
            let (start, nodes) = build_map_flow(size);
            group.bench_with_input(
                BenchmarkId::new(if with_trace { "trace_on" } else { "trace_off" }, size),
                &size,
                |b, _| {
                    let engine = ExecutionEngine::new();
                    b.iter(|| engine.execute_graph("bench", start, nodes.clone(), None, None, with_trace));
                },
            );
        }
    }
    group.finish();
}

/// Isolates `ValueStore::get` reference-resolution cost (Finding 3): a
/// struct field lookup by path, called repeatedly against a pre-populated
/// store, independent of the rest of the executor.
fn bench_value_store_get(c: &mut Criterion) {
    use taurus_core::runtime::execution::value_store::ValueStore;

    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        Value {
            kind: Some(Kind::StringValue("benchmark".to_string())),
        },
    );
    let flow_input = Value {
        kind: Some(Kind::StructValue(Struct { fields })),
    };
    let mut store = ValueStore::new(flow_input, false);
    store.insert_success_with_timing(1, int_value(42), Vec::new(), 0, 0);

    c.bench_function("value_store_get_by_node_id", |b| {
        b.iter(|| {
            store.get(&ReferenceValue {
                target: Some(reference_value::Target::NodeId(1)),
                paths: Vec::new(),
            })
        });
    });
}

criterion_group!(
    benches,
    bench_chain,
    bench_array_map,
    bench_value_store_get,
    bench_compile_vs_encode
);
criterion_main!(benches);

/// Gate-check for compiled-flow caching: is a correctness-safe cache key
/// even cheap? `NodeFunction`/`Value` only derive `PartialEq`, not `Hash`,
/// and `ExecutionFlow` has no version field, so the only correctness-safe
/// cache key is the flow's serialized content. Compares that serialization
/// cost against the full `execute_graph` (compile + run) cost it would be
/// competing against, for the same flow.
fn bench_compile_vs_encode(c: &mut Criterion) {
    use prost::Message as _;

    let (start, nodes) = build_chain_flow(200);
    let mut group = c.benchmark_group("compile_vs_encode");
    group.bench_function("encode_to_vec_cache_key/200", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            for n in &nodes {
                n.encode(&mut buf).unwrap();
            }
            buf
        });
    });
    group.bench_function("full_execute_graph/200 (for comparison)", |b| {
        let engine = ExecutionEngine::new();
        b.iter(|| engine.execute_graph("bench", start, nodes.clone(), None, None, false));
    });
    group.finish();
}
