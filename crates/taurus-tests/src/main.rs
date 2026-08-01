use std::path::Path;

use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tucana::shared::node_execution_result::{
    Id as NodeExecutionResultId, Result as NodeExecutionOutcome,
};
use tucana::shared::{
    NodeExecutionResult, ValidationFlow,
    helper::value::{from_json_value, to_json_value},
};

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
    #[serde(default)]
    pub remote: Option<RemoteFixture>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFixture {
    pub target_service: String,
    pub function_identifier: String,
    pub result_parameter: String,
}

struct FixtureRemoteRuntime {
    fixture: RemoteFixture,
}

#[async_trait::async_trait]
impl RemoteRuntime for FixtureRemoteRuntime {
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        if execution.target_service != self.fixture.target_service {
            return Err(RuntimeError::new(
                "T-TEST-000001",
                "UnexpectedRemoteService",
                format!(
                    "Expected remote service {}, received {}",
                    self.fixture.target_service, execution.target_service
                ),
            ));
        }
        if execution.request.function_identifier != self.fixture.function_identifier {
            return Err(RuntimeError::new(
                "T-TEST-000002",
                "UnexpectedRemoteFunction",
                format!(
                    "Expected remote function {}, received {}",
                    self.fixture.function_identifier, execution.request.function_identifier
                ),
            ));
        }

        let value = execution
            .request
            .parameters
            .as_ref()
            .and_then(|parameters| parameters.fields.get(&self.fixture.result_parameter))
            .cloned()
            .ok_or_else(|| {
                RuntimeError::new(
                    "T-TEST-000003",
                    "RemoteParameterMissing",
                    format!(
                        "Remote parameter {} was not provided",
                        self.fixture.result_parameter
                    ),
                )
            })?;

        Ok(NodeExecutionResult {
            started_at: 0,
            finished_at: 0,
            parameter_results: Vec::new(),
            id: Some(NodeExecutionResultId::FunctionIdentifier(
                execution.request.function_identifier,
            )),
            result: Some(NodeExecutionOutcome::Success(value)),
        })
    }
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

fn run_tests(cases: Cases) {
    for case in &cases.cases {
        match case.run() {
            CaseResult::Success => print_success(case),
            CaseResult::Failure(input, result) => print_failure(case, &input, result),
        }
    }
}

impl Testable for Case {
    fn run(&self) -> CaseResult {
        let engine = ExecutionEngine::new();
        let remote = self
            .remote
            .clone()
            .map(|fixture| FixtureRemoteRuntime { fixture });

        for input in self.inputs.clone() {
            let flow_input = input.clone().input.map(from_json_value);
            let (res, _) = engine.execute_graph(
                self.flow.starting_node_id,
                self.flow.node_functions.clone(),
                flow_input,
                remote.as_ref().map(|runtime| runtime as &dyn RemoteRuntime),
                None,
                false,
            );

            match res {
                taurus_core::types::signal::Signal::Failure(err) => {
                    let json = json!({
                        "name": err.category,
                        "message": err.message,
                    });
                    if json != input.clone().expected_result {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::types::signal::Signal::Success(value) => {
                    let json = to_json_value(value);
                    if json != input.clone().expected_result {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::types::signal::Signal::Return(value) => {
                    let json = to_json_value(value);
                    if json != input.clone().expected_result {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::types::signal::Signal::Respond(value) => {
                    let json = to_json_value(value);
                    if json != input.clone().expected_result {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::types::signal::Signal::Stop => continue,
            }
        }

        CaseResult::Success
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cases = Cases::from_path("./flows/");
    run_tests(cases);
}
