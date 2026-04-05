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
