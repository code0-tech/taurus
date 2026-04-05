use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use taurus_core::context::{context::Context, executor::Executor, registry::FunctionStore};
use tests_core::{Case, CaseResult, Cases, print_failure, print_success};

use tucana::shared::{
    NodeFunction,
    helper::value::{from_json_value, to_json_value},
};

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
        let store = FunctionStore::default();

        let node_functions: HashMap<i64, NodeFunction> = self
            .clone()
            .flow
            .node_functions
            .into_iter()
            .map(|node| (node.database_id, node))
            .collect();

        for input in self.inputs.clone() {
            let mut context = match input.clone().input {
                Some(inp) => Context::new(from_json_value(inp)),
                None => Context::default(),
            };

            let res = Executor::new(&store, node_functions.clone()).execute(
                self.flow.starting_node_id,
                &mut context,
                false,
            );

            match res {
                taurus_core::context::signal::Signal::Failure(err) => {
                    let json = json!({
                        "name": err.name,
                        "message": err.message,
                    });
                    return CaseResult::Failure(input, json);
                }
                taurus_core::context::signal::Signal::Success(value) => {
                    let json = to_json_value(value);
                    if json == input.clone().expected_result {
                        return CaseResult::Success;
                    } else {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::context::signal::Signal::Return(value) => {
                    let json = to_json_value(value);
                    if json == input.clone().expected_result {
                        return CaseResult::Success;
                    } else {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::context::signal::Signal::Respond(value) => {
                    let json = to_json_value(value);
                    if json == input.clone().expected_result {
                        return CaseResult::Success;
                    } else {
                        return CaseResult::Failure(input, json);
                    }
                }
                taurus_core::context::signal::Signal::Stop => continue,
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
