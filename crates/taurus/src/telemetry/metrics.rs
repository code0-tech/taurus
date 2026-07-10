use std::sync::OnceLock;

use opentelemetry::{
    KeyValue,
    metrics::{Counter, Histogram},
};
use tucana::shared::{
    NodeExecutionResult, node_execution_result::Id as NodeExecutionResultId,
    node_execution_result::Result as NodeExecutionOutcome,
};

static METRICS: OnceLock<Metrics> = OnceLock::new();

struct Metrics {
    flow_executions: Counter<u64>,
    flow_execution_duration: Histogram<f64>,
    function_executions: Counter<u64>,
    function_execution_duration: Histogram<f64>,
    function_failures: Counter<u64>,
}

pub fn initialize() {
    let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));
    let _ = METRICS.set(Metrics {
        flow_executions: meter.u64_counter("taurus.flow.executions").build(),
        flow_execution_duration: meter
            .f64_histogram("taurus.flow.execution.duration")
            .with_unit("s")
            .build(),
        function_executions: meter.u64_counter("taurus.function.executions").build(),
        function_execution_duration: meter
            .f64_histogram("taurus.function.execution.duration")
            .with_unit("s")
            .build(),
        function_failures: meter.u64_counter("taurus.function.failures").build(),
    });
}

pub struct FlowExecution<'a> {
    pub flow_id: i64,
    pub project_id: i64,
    pub flow_type: &'a str,
    pub outcome: &'static str,
    pub duration_seconds: f64,
}

pub fn flow_execution(execution: FlowExecution<'_>) {
    if let Some(metrics) = METRICS.get() {
        let attributes = flow_attributes(
            execution.flow_id,
            execution.project_id,
            execution.flow_type,
            execution.outcome,
        );
        metrics.flow_executions.add(1, &attributes);
        metrics
            .flow_execution_duration
            .record(execution.duration_seconds, &attributes);
    }
}

pub struct FunctionExecution<'a> {
    pub flow_id: i64,
    pub project_id: i64,
    pub flow_type: &'a str,
    pub function_identifier: &'a str,
    pub node_id: Option<i64>,
    pub outcome: &'static str,
    pub duration_seconds: f64,
    pub error_code: Option<&'a str>,
    pub error_category: Option<&'a str>,
}

pub fn function_execution(execution: FunctionExecution<'_>) {
    if let Some(metrics) = METRICS.get() {
        let attributes = function_attributes(&execution);
        metrics.function_executions.add(1, &attributes);
        metrics
            .function_execution_duration
            .record(execution.duration_seconds, &attributes);

        if execution.outcome == "failure" {
            let failure_attributes = function_failure_attributes(&execution);
            metrics.function_failures.add(1, &failure_attributes);
        }
    }
}

pub fn node_result_outcome(result: &NodeExecutionResult) -> &'static str {
    match result.result {
        Some(NodeExecutionOutcome::Success(_)) => "success",
        Some(NodeExecutionOutcome::Error(_)) => "failure",
        None => "missing",
    }
}

pub fn node_result_error(result: &NodeExecutionResult) -> (Option<&str>, Option<&str>) {
    match &result.result {
        Some(NodeExecutionOutcome::Error(error)) => {
            (Some(error.code.as_str()), Some(error.category.as_str()))
        }
        _ => (None, None),
    }
}

pub fn result_node_id(result: &NodeExecutionResult) -> Option<i64> {
    match result.id {
        Some(NodeExecutionResultId::NodeId(id)) => Some(id),
        _ => None,
    }
}

pub fn result_function_identifier(result: &NodeExecutionResult) -> Option<&str> {
    match &result.id {
        Some(NodeExecutionResultId::FunctionIdentifier(identifier)) => Some(identifier.as_str()),
        _ => None,
    }
}

pub fn duration_seconds(started_at_micros: i64, finished_at_micros: i64) -> f64 {
    let duration_micros = finished_at_micros.saturating_sub(started_at_micros);
    duration_micros as f64 / 1_000_000.0
}

fn flow_attributes(
    flow_id: i64,
    project_id: i64,
    flow_type: &str,
    outcome: &'static str,
) -> [KeyValue; 4] {
    [
        KeyValue::new("flow.id", flow_id),
        KeyValue::new("project.id", project_id),
        KeyValue::new("flow.type", flow_type.to_owned()),
        KeyValue::new("outcome", outcome),
    ]
}

fn function_attributes(execution: &FunctionExecution<'_>) -> Vec<KeyValue> {
    let mut attributes = vec![
        KeyValue::new("flow.id", execution.flow_id),
        KeyValue::new("project.id", execution.project_id),
        KeyValue::new("flow.type", execution.flow_type.to_owned()),
        KeyValue::new(
            "function.identifier",
            execution.function_identifier.to_owned(),
        ),
        KeyValue::new("outcome", execution.outcome),
    ];
    if let Some(node_id) = execution.node_id {
        attributes.push(KeyValue::new("node.id", node_id));
    }
    attributes
}

fn function_failure_attributes(execution: &FunctionExecution<'_>) -> Vec<KeyValue> {
    let mut attributes = function_attributes(execution);
    if let Some(error_code) = execution.error_code {
        attributes.push(KeyValue::new("error.code", error_code.to_owned()));
    }
    if let Some(error_category) = execution.error_category {
        attributes.push(KeyValue::new("error.category", error_category.to_owned()));
    }
    attributes
}
