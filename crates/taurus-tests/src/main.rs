//! Batch fixture runner: loads every `ValidationFlow` fixture under `./flows/`
//! (see `taurus_core::fixtures`), runs each through `ExecutionEngine::execute_graph`
//! directly (no NATS involved), and logs a pass/fail line per case. Not wired
//! into `cargo test`; run explicitly via `cargo run --package taurus-tests`.

use serde_json::json;
use taurus_core::fixtures::{Case, Cases, Input, RemoteFixture, print_failure, print_success};
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tucana::shared::node_execution_result::{
    Id as NodeExecutionResultId, Result as NodeExecutionOutcome,
};
use tucana::shared::{
    NodeExecutionResult,
    helper::value::{from_json_value, to_json_value},
};

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
