use core::context::{context::Context, executor::Executor, registry::FunctionStore};
use log::{error, info};
use serde::Deserialize;
use std::collections::HashMap;

use tucana::shared::NodeFunction;

#[derive(Clone, Deserialize)]
struct Input {
    input: Option<tucana::shared::Value>,
    expected_result: serde_json::Value,
}

#[derive(Clone, Deserialize)]
struct Case {
    name: String,
    description: String,
    inputs: Vec<Input>,
    flow: TestableFlow,
}

#[derive(Clone, Deserialize)]
struct TestableFlow {
    pub starting_node_id: i64,
    pub node_functions: Vec<NodeFunction>,
}

#[derive(Clone, Deserialize)]
struct TestCases {
    cases: Vec<Case>,
}

fn print_success(case: &Case) {
    info!("test {} ... ok", case.name);
}

fn print_failure(case: &Case, input: &Input) {
    error!("test {} ... FAILED", case.name);
    error!("  input: {:?}", input.input);
    error!("  expected: {:?}", input.expected_result);
    error!("  message: {}", case.description);
}

fn get_test_cases(path: &str) -> TestCases {
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
        let path = entry.path();

        let content = match std::fs::read_to_string(&path) {
            Ok(it) => it,
            Err(err) => {
                log::error!("Cannot read file ({:?}): {:?}", path, err);
                continue;
            }
        };
        items.push(match serde_json::from_str(&content) {
            Ok(it) => it,
            Err(err) => {
                log::error!("Cannot read json ({:?}): {:?}", path, err);
                continue;
            }
        });
    }

    TestCases { cases: items }
}

impl TestCases {
    pub fn from_path(path: &str) -> Self {
        get_test_cases(path)
    }

    pub fn run_tests(&self) {
        for case in self.cases.clone() {
            match case.run() {
                CaseResult::Success => print_success(&case),
                CaseResult::Failure(input) => print_failure(&case, &input),
            }
        }
    }
}

enum CaseResult {
    Success,
    Failure(Input),
}

impl Case {
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
                Some(inp) => Context::new(inp),
                None => Context::default(),
            };

            let res = Executor::new(&store, node_functions.clone())
                .execute(self.flow.starting_node_id, &mut context);

            match res {
                core::context::signal::Signal::Failure(_) => {
                    return CaseResult::Failure(input);
                }
                _ => continue,
            }
        }

        CaseResult::Success
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();


    let cases = TestCases::from_path("./crates/tests/flows/");
    cases.run_tests();
}
