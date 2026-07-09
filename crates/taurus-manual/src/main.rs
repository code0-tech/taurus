use std::path::Path;
use std::time::{Duration, Instant};

use clap::Parser;
use log::error;
use log::info;
use prost::Message;
use serde::Deserialize;
use taurus_core::runtime::engine::{ExecutionEngine, ExecutionId};
use taurus_core::time::now_unix_micros;
use taurus_core::types::signal::Signal;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tucana::shared::ExecutionFlow;
use tucana::shared::NodeExecutionResult;
use tucana::shared::ValidationFlow;
use tucana::shared::Value;
use tucana::shared::helper::value::from_json_value;
use tucana::shared::helper::value::to_json_value;
use tucana::shared::node_execution_result::Id as NodeExecutionResultId;
use tucana::shared::node_execution_result::Result as NodeExecutionResultResult;
use tucana::shared::value::Kind;

#[derive(Clone, Deserialize)]
pub struct Input {
    pub input: Option<serde_json::Value>,
    pub expected_result: serde_json::Value,
}

#[derive(Clone, Deserialize)]
pub struct Case {
    pub name: String,
    pub description: String,
    pub inputs: Vec<Input>,
    pub flow: ValidationFlow,
}

pub enum CaseResult {
    Success,
    Failure(Input, serde_json::Value),
}

pub trait Testable {
    fn run(&self) -> CaseResult;
}

#[derive(Clone, Deserialize)]
pub struct Cases {
    pub cases: Vec<Case>,
}

pub fn print_success(case: &Case) {
    info!("test {} ... ok", case.name);
}

pub fn print_failure(case: &Case, input: &Input, result: serde_json::Value) {
    error!("test {} ... FAILED", case.name);
    error!("  input: {:?}", input.input);
    error!("  expected: {:?}", input.expected_result);
    error!("  real_value: {:?}", result);
    error!("  message: {}", case.description);
}

fn get_test_case<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Option<Case> {
    let content = match std::fs::read_to_string(&path) {
        Ok(it) => it,
        Err(err) => {
            log::error!("Cannot read file ({:?}): {:?}", path, err);
            return None;
        }
    };

    match serde_json::from_str(&content) {
        Ok(it) => it,
        Err(err) => {
            log::error!("Cannot read json ({:?}): {:?}", path, err);
            None
        }
    }
}

fn get_test_cases(path: &str) -> Cases {
    let mut items = Vec::new();
    let dir = match std::fs::read_dir(path) {
        Ok(d) => d,
        Err(err) => {
            panic!("Cannot open path: {:?}", err)
        }
    };

    for entry in dir {
        let entry = match entry {
            Ok(it) => it,
            Err(err) => {
                log::error!("Cannot read entry: {:?}", err);
                continue;
            }
        };
        let file_path = entry.path();
        items.push(match get_test_case(&file_path) {
            Some(it) => it,
            None => {
                continue;
            }
        });
    }

    Cases { cases: items }
}

impl Case {
    pub fn from_path(path: &str) -> Self {
        match get_test_case(path) {
            Some(s) => s,
            None => panic!("flow was not found"),
        }
    }
}

impl Cases {
    pub fn from_path(path: &str) -> Self {
        get_test_cases(path)
    }
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Index value
    #[arg(short, long, default_value_t = 0)]
    index: i32,

    /// NATS server URL
    #[arg(short, long, default_value_t = String::from("nats://127.0.0.1:4222"))]
    nats_url: String,

    /// Path value
    #[arg(short, long)]
    path: String,

    /// Queue the selected flow on a running Taurus instance instead of executing locally
    #[arg(long, default_value_t = false)]
    queue_execution: bool,

    /// Execute locally without connecting to NATS remote runtime or emitter
    #[arg(long, default_value_t = false)]
    offline: bool,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();
    let index = args.index;
    let nats_url = args.nats_url;
    let path = args.path;
    let case = Case::from_path(&path);

    if args.offline && args.queue_execution {
        panic!("--offline cannot be combined with --queue-execution");
    }

    let flow_input = match case.inputs.get(index as usize) {
        Some(inp) => inp.input.clone().map(from_json_value),
        None => None,
    };

    if args.offline {
        let engine = ExecutionEngine::new();
        let started_at = now_unix_micros();
        let start = Instant::now();
        let report = engine.execute_graph_report(
            case.flow.starting_node_id,
            case.flow.node_functions.clone(),
            flow_input,
            None,
            None,
            true,
        );
        let duration_us = start.elapsed().as_micros();
        let finished_at = now_unix_micros();
        print_manual_execution_debug(
            started_at,
            finished_at,
            duration_us,
            &report.node_execution_results,
        );
        print_signal(report.signal);
        return;
    }

    let client = match async_nats::connect(nats_url).await {
        Ok(client) => {
            log::info!("Connected to nats server");
            client
        }
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    };

    if args.queue_execution {
        queue_execution(&client, &case, flow_input).await;
        return;
    }

    let remote =
        NATSRemoteRuntime::with_execution_result_timeout(client.clone(), Duration::from_secs(30));
    let emitter = NATSRespondEmitter::new(client);
    let engine = ExecutionEngine::new();

    let started_at = now_unix_micros();
    let start = Instant::now();
    let report = engine.execute_graph_report(
        case.flow.starting_node_id,
        case.flow.node_functions.clone(),
        flow_input,
        Some(&remote),
        Some(&emitter),
        false,
    );
    let duration_us = start.elapsed().as_micros();
    let finished_at = now_unix_micros();
    print_manual_execution_debug(
        started_at,
        finished_at,
        duration_us,
        &report.node_execution_results,
    );
    emitter.shutdown().await;

    print_signal(report.signal);
}

fn print_signal(signal: Signal) {
    match signal {
        Signal::Success(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        Signal::Return(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        Signal::Respond(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        Signal::Stop => println!("Received Stop signal"),
        Signal::Failure(runtime_error) => {
            println!("RuntimeError: {:?}", runtime_error);
        }
    }
}

async fn queue_execution(
    client: &async_nats::Client,
    case: &Case,
    input_value: Option<tucana::shared::Value>,
) {
    let execution_id = ExecutionId::new_v4();
    let execution_flow = ExecutionFlow {
        flow_id: case.flow.flow_id,
        project_id: case.flow.project_id,
        starting_node_id: case.flow.starting_node_id,
        node_functions: case.flow.node_functions.clone(),
        input_value,
    };
    let execution_topic = format!("execution.{}", execution_id);

    info!(
        "Queueing execution of flow {} with execution id {}",
        execution_flow.flow_id, execution_id
    );

    if let Err(err) = client
        .publish(
            execution_topic.clone(),
            execution_flow.encode_to_vec().into(),
        )
        .await
    {
        panic!(
            "Failed to publish flow {} to execution topic '{}': {}",
            case.flow.flow_id, execution_topic, err
        );
    }

    if let Err(err) = client.flush().await {
        panic!(
            "Failed to flush execution request on '{}': {}",
            execution_topic, err
        );
    }

    println!("{}", execution_id);
}

fn print_manual_execution_debug(
    started_at: i64,
    finished_at: i64,
    duration_us: u128,
    node_results: &[NodeExecutionResult],
) {
    let mut normalized_results = node_results.to_vec();
    normalize_node_execution_results(&mut normalized_results);
    print_timing_debug(started_at, finished_at, duration_us, &normalized_results);
    print_execution_result_debug(&normalized_results);
}

fn print_timing_debug(
    started_at: i64,
    finished_at: i64,
    duration_us: u128,
    node_results: &[NodeExecutionResult],
) {
    eprintln!("[manual timing] unit=microseconds");
    eprintln!("[manual timing] started_at_unix_us={}", started_at);
    eprintln!("[manual timing] finished_at_unix_us={}", finished_at);
    eprintln!("[manual timing] wall_delta_us={}", finished_at - started_at);
    eprintln!("[manual timing] instant_duration_us={}", duration_us);
    eprintln!("[manual timing] node_count={}", node_results.len());

    for (execution_index, result) in node_results.iter().enumerate() {
        let params = result
            .parameter_results
            .iter()
            .enumerate()
            .map(|(param_index, param)| {
                let value = param
                    .value
                    .clone()
                    .map(to_json_value)
                    .unwrap_or(serde_json::Value::Null);
                format!("{}={}", param_index, value)
            })
            .collect::<Vec<_>>()
            .join(", ");

        eprintln!(
            "[manual timing] execution_index={} {} started_at_unix_us={} finished_at_unix_us={} delta_us={} params=[{}]",
            execution_index,
            execution_result_id_label(result),
            result.started_at,
            result.finished_at,
            result.finished_at - result.started_at,
            params
        );
    }
}

fn print_execution_result_debug(node_results: &[NodeExecutionResult]) {
    eprintln!("[manual execution result] {:#?}", node_results);
}

fn normalize_node_execution_results(node_results: &mut [NodeExecutionResult]) {
    for result in node_results {
        normalize_node_execution_result(result);
    }
}

fn normalize_node_execution_result(result: &mut NodeExecutionResult) {
    for parameter_result in &mut result.parameter_results {
        match &mut parameter_result.value {
            Some(value) => normalize_value(value),
            None => {
                parameter_result.value = Some(null_value());
            }
        }
    }

    match &mut result.result {
        Some(NodeExecutionResultResult::Success(value)) => normalize_value(value),
        Some(NodeExecutionResultResult::Error(error)) => {
            if let Some(details) = &mut error.details {
                for value in details.fields.values_mut() {
                    normalize_value(value);
                }
            }
        }
        None => {
            result.result = Some(NodeExecutionResultResult::Success(null_value()));
        }
    }
}

fn normalize_value(value: &mut Value) {
    match &mut value.kind {
        Some(Kind::StructValue(struct_value)) => {
            for field in struct_value.fields.values_mut() {
                normalize_value(field);
            }
        }
        Some(Kind::ListValue(list_value)) => {
            for item in &mut list_value.values {
                normalize_value(item);
            }
        }
        Some(Kind::NumberValue(number)) if number.number.is_none() => {
            value.kind = Some(Kind::NullValue(0));
        }
        Some(_) => {}
        None => {
            value.kind = Some(Kind::NullValue(0));
        }
    }
}

fn null_value() -> Value {
    Value {
        kind: Some(Kind::NullValue(0)),
    }
}

fn execution_result_id_label(result: &NodeExecutionResult) -> String {
    match &result.id {
        Some(NodeExecutionResultId::NodeId(id)) => format!("node_id={}", id),
        Some(NodeExecutionResultId::FunctionIdentifier(id)) => {
            format!("function_identifier={}", id)
        }
        None => "id=<missing>".to_string(),
    }
}
