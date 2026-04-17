use super::*;
use crate::context::registry::{HandlerFn, IntoFunctionEntry};
use std::cell::RefCell;
use tucana::shared::number_value::Number;
use tucana::shared::{NodeFunction, NumberValue, ReferenceValue};

fn num(i: i64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(NumberValue {
            number: Some(Number::Integer(i)),
        })),
    }
}

fn extract_i64(v: &Value) -> i64 {
    match &v.kind {
        Some(Kind::NumberValue(NumberValue {
            number: Some(Number::Integer(i)),
        })) => *i,
        other => panic!("Expected integer number value, got {:?}", other),
    }
}

fn node(id: i64, function_id: &str, next: Option<i64>) -> NodeFunction {
    NodeFunction {
        database_id: id,
        runtime_function_id: function_id.to_string(),
        definition_source: "taurus".to_string(),
        next_node_id: next,
        ..Default::default()
    }
}

fn node_reference(node_id: i64) -> ReferenceValue {
    ReferenceValue {
        paths: vec![],
        target: Some(Target::NodeId(node_id)),
    }
}

fn assert_node_success(ctx: &mut Context, node_id: i64, expected: i64) {
    match ctx.get(node_reference(node_id)) {
        ContextResult::Success(v) => assert_eq!(extract_i64(&v), expected),
        ContextResult::Error(_) => {
            panic!("Expected success result for node {}, got error", node_id)
        }
        ContextResult::NotFound => {
            panic!(
                "Expected success result for node {}, got not found",
                node_id
            )
        }
    }
}

fn assert_node_error(ctx: &mut Context, node_id: i64, expected_name: &str) {
    match ctx.get(node_reference(node_id)) {
        ContextResult::Error(err) => assert_eq!(err.name, expected_name),
        ContextResult::Success(_) => {
            panic!("Expected error result for node {}, got success", node_id)
        }
        ContextResult::NotFound => {
            panic!("Expected error result for node {}, got not found", node_id)
        }
    }
}

fn assert_node_not_found(ctx: &mut Context, node_id: i64) {
    match ctx.get(node_reference(node_id)) {
        ContextResult::NotFound => {}
        ContextResult::Success(_) => {
            panic!("Expected no result for node {}, got success", node_id)
        }
        ContextResult::Error(_) => {
            panic!("Expected no result for node {}, got error", node_id)
        }
    }
}

fn success_one_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Success(num(1))
}

fn success_two_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Success(num(2))
}

fn success_three_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Success(num(3))
}

fn failure_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Failure(RuntimeError::simple_str("TestFailure", "expected failure"))
}

fn return_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Return(num(9))
}

fn stop_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Stop
}

fn respond_eleven_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Respond(num(11))
}

fn respond_twentytwo_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Respond(num(22))
}

fn respond_fortyfour_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Respond(num(44))
}

fn return_seventyseven_handler(
    _args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    Signal::Return(num(77))
}

fn runtime_call_child_two_handler(
    _args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    match run(2, ctx) {
        Signal::Success(_) => Signal::Success(num(55)),
        other => other,
    }
}

fn runtime_call_child_return_handler(
    _args: &[Argument],
    ctx: &mut Context,
    run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    run(2, ctx)
}

#[test]
fn success_signal_continues_through_next_nodes() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::success_one",
            HandlerFn::eager(success_one_handler, 0),
        ),
        (
            "test::success_two",
            HandlerFn::eager(success_two_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::success_one", Some(2)));
    nodes.insert(2, node(2, "test::success_two", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    match signal {
        Signal::Success(v) => assert_eq!(extract_i64(&v), 2),
        other => panic!("Expected success signal, got {:?}", other),
    }

    assert_node_success(&mut ctx, 1, 1);
    assert_node_success(&mut ctx, 2, 2);
}

#[test]
fn failure_signal_stops_flow_and_persists_error() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::success_one",
            HandlerFn::eager(success_one_handler, 0),
        ),
        ("test::failure", HandlerFn::eager(failure_handler, 0)),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::success_one", Some(2)));
    nodes.insert(2, node(2, "test::failure", Some(3)));
    nodes.insert(3, node(3, "test::success_three", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    match signal {
        Signal::Failure(err) => assert_eq!(err.name, "TestFailure"),
        other => panic!("Expected failure signal, got {:?}", other),
    }

    assert_node_success(&mut ctx, 1, 1);
    assert_node_error(&mut ctx, 2, "TestFailure");
    assert_node_not_found(&mut ctx, 3);
}

#[test]
fn return_signal_breaks_execution_and_skips_next_nodes() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        ("test::return", HandlerFn::eager(return_handler, 0)),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::return", Some(2)));
    nodes.insert(2, node(2, "test::success_three", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    match signal {
        Signal::Return(v) => assert_eq!(extract_i64(&v), 9),
        other => panic!("Expected return signal, got {:?}", other),
    }

    assert_node_not_found(&mut ctx, 1);
    assert_node_not_found(&mut ctx, 2);
}

#[test]
fn stop_signal_stops_execution_and_skips_next_nodes() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        ("test::stop", HandlerFn::eager(stop_handler, 0)),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::stop", Some(2)));
    nodes.insert(2, node(2, "test::success_three", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    assert!(matches!(signal, Signal::Stop));
    assert_node_not_found(&mut ctx, 1);
    assert_node_not_found(&mut ctx, 2);
}

#[test]
fn respond_signal_emits_for_each_node_and_execution_continues() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::respond_eleven",
            HandlerFn::eager(respond_eleven_handler, 0),
        ),
        (
            "test::respond_twentytwo",
            HandlerFn::eager(respond_twentytwo_handler, 0),
        ),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::respond_eleven", Some(2)));
    nodes.insert(2, node(2, "test::respond_twentytwo", Some(3)));
    nodes.insert(3, node(3, "test::success_three", None));

    let emitted = RefCell::new(Vec::<Value>::new());
    let emitter = |v: Value| {
        emitted.borrow_mut().push(v);
    };

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes)
        .with_respond_emitter(&emitter)
        .execute(1, &mut ctx, false);

    match signal {
        Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
        other => panic!("Expected final success signal, got {:?}", other),
    }

    let emitted_values = emitted.borrow();
    assert_eq!(emitted_values.len(), 2);
    assert_eq!(extract_i64(&emitted_values[0]), 11);
    assert_eq!(extract_i64(&emitted_values[1]), 22);

    assert_node_success(&mut ctx, 1, 11);
    assert_node_success(&mut ctx, 2, 22);
    assert_node_success(&mut ctx, 3, 3);
}

#[test]
fn respond_signal_emits_inside_runtime_call_and_parent_continues() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::runtime_call_child_two",
            HandlerFn::eager(runtime_call_child_two_handler, 0),
        ),
        (
            "test::respond_fortyfour",
            HandlerFn::eager(respond_fortyfour_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::runtime_call_child_two", None));
    nodes.insert(2, node(2, "test::respond_fortyfour", None));

    let emitted = RefCell::new(Vec::<Value>::new());
    let emitter = |v: Value| {
        emitted.borrow_mut().push(v);
    };

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes)
        .with_respond_emitter(&emitter)
        .execute(1, &mut ctx, false);

    match signal {
        Signal::Success(v) => assert_eq!(extract_i64(&v), 55),
        other => panic!("Expected final success signal, got {:?}", other),
    }

    let emitted_values = emitted.borrow();
    assert_eq!(emitted_values.len(), 1);
    assert_eq!(extract_i64(&emitted_values[0]), 44);

    assert_node_success(&mut ctx, 1, 55);
    assert_node_success(&mut ctx, 2, 44);
}

#[test]
fn return_signal_in_child_context_continues_parent_flow() {
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::runtime_call_child_return",
            HandlerFn::eager(runtime_call_child_return_handler, 0),
        ),
        (
            "test::return_seventyseven",
            HandlerFn::eager(return_seventyseven_handler, 0),
        ),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::runtime_call_child_return", Some(3)));
    nodes.insert(2, node(2, "test::return_seventyseven", None));
    nodes.insert(3, node(3, "test::success_three", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    match signal {
        Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
        other => panic!("Expected final success signal, got {:?}", other),
    }

    // Child return exits its own context and is treated as value by parent node.
    assert_node_success(&mut ctx, 1, 77);
    assert_node_not_found(&mut ctx, 2);
    assert_node_success(&mut ctx, 3, 3);
}

#[test]
fn child_return_skips_child_next_but_parent_next_runs() {
    // Graph:
    // node1 -> next node4, and node1 executes child node2 via run(...)
    // node2 -> RETURN and has next node3
    // node3 must NOT run (because node2 returned)
    // node4 must run (parent chain continues)
    let mut store = FunctionStore::new();
    store.populate(vec![
        (
            "test::runtime_call_child_return",
            HandlerFn::eager(runtime_call_child_return_handler, 0),
        ),
        (
            "test::return_seventyseven",
            HandlerFn::eager(return_seventyseven_handler, 0),
        ),
        (
            "test::success_two",
            HandlerFn::eager(success_two_handler, 0),
        ),
        (
            "test::success_three",
            HandlerFn::eager(success_three_handler, 0),
        ),
    ]);

    let mut nodes = HashMap::new();
    nodes.insert(1, node(1, "test::runtime_call_child_return", Some(4)));
    nodes.insert(2, node(2, "test::return_seventyseven", Some(3)));
    nodes.insert(3, node(3, "test::success_two", None));
    nodes.insert(4, node(4, "test::success_three", None));

    let mut ctx = Context::default();
    let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

    match signal {
        Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
        other => panic!("Expected final success signal, got {:?}", other),
    }

    // node1 ran and received child return value
    assert_node_success(&mut ctx, 1, 77);
    // node2 returned, so it has no success/error entry in context
    assert_node_not_found(&mut ctx, 2);
    // node3 is child next node and must be skipped due to return in node2
    assert_node_not_found(&mut ctx, 3);
    // parent next node still executes
    assert_node_success(&mut ctx, 4, 3);
}
