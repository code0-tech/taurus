//! Batch fixture runner: runs every `ValidationFlow` fixture of two suites
//! (see `taurus_core::fixtures`) through `ExecutionEngine::execute_graph`
//! directly (no NATS involved) and logs a pass/fail line per case:
//!
//! - `flows/local/`: flows that never leave Taurus. They run without a remote
//!   runtime, so a flow that tries to call an action fails as it would
//!   without one configured.
//! - `flows/remote/`: flows that call actions. They run against
//!   [`SimulatedActions`], an in-process stand-in for a few action functions.
//!
//! Not wired into `cargo test`; run explicitly via
//! `cargo run --package taurus-tests [-- local|remote]`. Exits non-zero if
//! any case fails, so CI catches regressions.

mod simulated_actions;

use serde_json::json;
use taurus_core::fixtures::{Case, Cases, Input, print_failure, print_success};
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::runtime::remote::RemoteRuntime;
use taurus_core::types::signal::Signal;
use tucana::shared::helper::value::{from_json_value, to_json_value};

use simulated_actions::SimulatedActions;

#[derive(Clone, Copy, PartialEq)]
enum Suite {
    Local,
    Remote,
}

impl Suite {
    const ALL: [Suite; 2] = [Suite::Local, Suite::Remote];

    fn name(self) -> &'static str {
        match self {
            Suite::Local => "local",
            Suite::Remote => "remote",
        }
    }

    fn path(self) -> &'static str {
        match self {
            Suite::Local => "./flows/local/",
            Suite::Remote => "./flows/remote/",
        }
    }
}

pub enum CaseResult {
    Success,
    Failure(Input, serde_json::Value),
}

/// Runs every case of `suite` and returns how many failed.
fn run_suite(suite: Suite) -> usize {
    let cases = Cases::from_path(suite.path());
    log::info!(
        "running {} suite ({} cases)",
        suite.name(),
        cases.cases.len()
    );

    let mut failed = 0;
    for case in &cases.cases {
        match run_case(case, suite) {
            CaseResult::Success => print_success(case),
            CaseResult::Failure(input, result) => {
                failed += 1;
                print_failure(case, &input, result);
            }
        }
    }
    failed
}

fn run_case(case: &Case, suite: Suite) -> CaseResult {
    let engine = ExecutionEngine::new();
    let simulated_actions = SimulatedActions::new(&engine);
    let remote = (suite == Suite::Remote).then_some(&simulated_actions as &dyn RemoteRuntime);

    for input in case.inputs.clone() {
        let flow_input = input.clone().input.map(from_json_value);
        let (signal, _) = engine.execute_graph(
            "test-execution",
            case.flow.starting_node_id,
            case.flow.node_functions.clone(),
            flow_input,
            remote,
            false,
        );

        let result = match signal {
            Signal::Failure(err) => json!({
                "name": err.category,
                "message": err.message,
            }),
            Signal::Success(value) | Signal::Return(value) => to_json_value(value),
            Signal::Stop => continue,
        };
        if result != input.expected_result {
            return CaseResult::Failure(input, result);
        }
    }

    CaseResult::Success
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let suites: Vec<Suite> = match std::env::args().nth(1).as_deref() {
        None => Suite::ALL.to_vec(),
        Some("local") => vec![Suite::Local],
        Some("remote") => vec![Suite::Remote],
        Some(other) => {
            log::error!("unknown suite {:?}; expected `local` or `remote`", other);
            std::process::exit(2);
        }
    };

    let failed: usize = suites.into_iter().map(run_suite).sum();
    if failed > 0 {
        log::error!("{} case(s) failed", failed);
        std::process::exit(1);
    }
}
