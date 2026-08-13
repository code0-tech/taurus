//! Batch fixture runner: loads every `ValidationFlow` fixture under `./flows/`
//! (see `taurus_core::fixtures`), runs each through `ExecutionEngine::execute_graph`
//! directly (no NATS involved), and logs a pass/fail line per case. Not wired
//! into `cargo test`; run explicitly via `cargo run --package taurus-tests`.

use serde_json::json;
use taurus_core::fixtures::{Case, Cases, Input, RemoteFixture, print_failure, print_success};
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use tucana::aquila::{ActionNodeSubFlowValue, action_node_value};
use tucana::shared::node_execution_result::{
    Id as NodeExecutionResultId, Result as NodeExecutionOutcome,
};
use tucana::shared::value::Kind;
use tucana::shared::{
    NodeExecutionResult, Value,
    helper::value::{from_json_value, to_json_value},
};

struct FixtureRemoteRuntime<'a> {
    fixture: RemoteFixture,
    // Needed to simulate an action invoking a minted `SubFlow` reference
    // back into `ExecutionEngine::execute_sub_flow` (see `sub_flow_calls`)
    // -- the same engine instance the flow itself is running on, so the
    // callback resolves against the registry entry the flow's own remote
    // call minted.
    engine: &'a ExecutionEngine,
}

#[async_trait::async_trait]
impl RemoteRuntime for FixtureRemoteRuntime<'_> {
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

        // A `SubFlow`-valued parameter means this request carries a minted
        // callback reference (see `resolve_remote_args`/`SubFlowRegistry`)
        // -- drive it via `sub_flow_calls` regardless of how many entries
        // that list has (zero is a legitimate, real case: an empty input
        // list means the reference is minted but never actually invoked).
        // Only fall through to the literal-echo path below when no
        // `SubFlow` parameter is present at all.
        if let Some(execution_identifier) =
            execution.request.parameters.iter().find_map(|parameter| {
                match parameter.value.as_ref()? {
                    action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                        execution_identifier,
                        ..
                    }) => Some(execution_identifier.clone()),
                    _ => None,
                }
            })
        {
            return self
                .drive_sub_flow_calls(&execution, &execution_identifier)
                .await;
        }

        // Parameters are positional on the wire now (no key), so fixtures
        // with a `resultParameter` only make sense with a single remote
        // parameter — take it directly rather than looking it up by name.
        let value = execution
            .request
            .parameters
            .first()
            .and_then(|parameter| parameter.value.as_ref())
            .and_then(|value| match value {
                // Fixtures never exercise `${signature}` templating, so the
                // literal's `value` is always already the concrete result.
                action_node_value::Value::LiteralValue(literal) => literal.value.clone(),
                action_node_value::Value::SubFlow(_) => None,
            })
            .ok_or_else(|| {
                RuntimeError::new(
                    "T-TEST-000003",
                    "RemoteParameterMissing",
                    format!(
                        "Remote parameter {:?} was not provided",
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

impl FixtureRemoteRuntime<'_> {
    /// Simulates an action driving `ActionSubFlowExecutionRequest` traffic
    /// against `execution_identifier` (the request's minted `SubFlow`
    /// reference): one `execute_sub_flow` call per `sub_flow_calls` entry,
    /// in order -- exactly as `hercules`'s `for_each` implementation calls
    /// back once per list element via `executeSubFlow`. An empty
    /// `sub_flow_calls` is valid and drives zero calls (e.g. an empty input
    /// list: the reference is minted but never actually invoked). Fails the
    /// whole remote call the moment any individual sub-flow run fails,
    /// since a real action would abort the same way. Resolves to `null`,
    /// matching a `void`-signature consumer-driving remote function (e.g.
    /// `for_each` itself).
    async fn drive_sub_flow_calls(
        &self,
        execution: &RemoteExecution,
        execution_identifier: &str,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        for call in &self.fixture.sub_flow_calls {
            let value = from_json_value(call.clone());
            let report = self
                .engine
                .execute_sub_flow(&execution_identifier, vec![value], Some(self), false)
                .await
                .ok_or_else(|| {
                    RuntimeError::new(
                        "T-TEST-000005",
                        "SubFlowNotFound",
                        format!(
                            "Sub flow {} was not minted (or already resolved)",
                            execution_identifier
                        ),
                    )
                })?;
            if let Signal::Failure(err) = report.signal {
                return Err(err);
            }
        }

        Ok(NodeExecutionResult {
            started_at: 0,
            finished_at: 0,
            parameter_results: Vec::new(),
            id: Some(NodeExecutionResultId::FunctionIdentifier(
                execution.request.function_identifier.clone(),
            )),
            result: Some(NodeExecutionOutcome::Success(Value {
                kind: Some(Kind::NullValue(0)),
            })),
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
        let remote = self.remote.clone().map(|fixture| FixtureRemoteRuntime {
            fixture,
            engine: &engine,
        });

        for input in self.inputs.clone() {
            let flow_input = input.clone().input.map(from_json_value);
            let (res, _) = engine.execute_graph(
                "test-execution",
                self.flow.starting_node_id,
                self.flow.node_functions.clone(),
                flow_input,
                remote.as_ref().map(|runtime| runtime as &dyn RemoteRuntime),
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
