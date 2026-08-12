//! JSON fixture loading shared by flow validation tooling binaries.
//!
//! `taurus-manual` and `taurus-tests` both load `ValidationFlow` fixtures
//! from JSON files and report pass/fail results in the same format; this
//! module is the single definition of that fixture format.

use std::path::Path;

use log::{error, info};
use serde::Deserialize;
use tucana::shared::ValidationFlow;

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
    /// Only meaningful when `sub_flow_calls` is empty: the literal-valued
    /// parameter to echo straight back as this remote call's own result
    /// (see `0012_remote_function_subflow.json`).
    #[serde(default)]
    pub result_parameter: Option<String>,
    /// Positional values to drive one `ExecutionEngine::execute_sub_flow`
    /// call per entry against this request's `SubFlow`-valued parameter, in
    /// order -- simulates an action invoking a minted sub-flow reference
    /// the same number of times a real action would (e.g. once per element
    /// for a remotely-dispatched `for_each`'s consumer callback). When
    /// non-empty, the remote call itself resolves to `null` once every call
    /// has run, mirroring a `void`-signature remote function.
    #[serde(default)]
    pub sub_flow_calls: Vec<serde_json::Value>,
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
            error!("Cannot read file ({:?}): {:?}", path, err);
            return None;
        }
    };

    match serde_json::from_str(&content) {
        Ok(it) => it,
        Err(err) => {
            error!("Cannot read json ({:?}): {:?}", path, err);
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
                error!("Cannot read entry: {:?}", err);
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
