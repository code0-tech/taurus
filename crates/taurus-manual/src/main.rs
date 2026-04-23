use std::path::Path;

use clap::{Parser, arg, command};
use log::error;
use log::info;
use serde::Deserialize;
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::types::signal::Signal;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tucana::shared::ValidationFlow;
use tucana::shared::helper::value::from_json_value;
use tucana::shared::helper::value::to_json_value;

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

    let flow_input = match case.inputs.get(index as usize) {
        Some(inp) => match inp.input.clone() {
            Some(json_input) => Some(from_json_value(json_input)),
            None => None,
        },
        None => None,
    };

    let client = match async_nats::connect(nats_url).await {
        Ok(client) => {
            log::info!("Connected to nats server");
            client
        }
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    };
    let remote = NATSRemoteRuntime::new(client.clone());
    let emitter = NATSRespondEmitter::new(client);
    let engine = ExecutionEngine::new();
    let (result, _) = engine.execute_graph(
        case.flow.starting_node_id,
        case.flow.node_functions.clone(),
        flow_input,
        Some(&remote),
        Some(&emitter),
        false,
    );
    emitter.shutdown().await;

    match result {
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
