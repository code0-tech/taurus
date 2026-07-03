use code0_flow::flow_service::{
    auth::get_authorization_metadata, retry::create_channel_with_retry,
};
use std::time::Duration;
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{ExecutionRequest, execution_service_client::ExecutionServiceClient},
    shared::{
        ExecutionResult, NodeExecutionResult, Value, execution_result, node_execution_result,
        value::Kind,
    },
};

pub struct TaurusRuntimeExecutionService {
    client: ExecutionServiceClient<Channel>,
    aquila_token: String,
}

impl TaurusRuntimeExecutionService {
    pub async fn from_url(
        aquila_url: String,
        aquila_token: String,
        connect_timeout: Duration,
        request_timeout: Duration,
    ) -> Self {
        let channel =
            create_channel_with_retry("Aquila", aquila_url, connect_timeout, request_timeout).await;
        let client = ExecutionServiceClient::new(channel);

        TaurusRuntimeExecutionService {
            client,
            aquila_token,
        }
    }

    pub async fn update_runtime_execution(&mut self, mut runtime_execution: ExecutionResult) {
        log::info!(
            "Transmitting execution result to Aquila (execution_id={}, flow_id={}, node_results={})",
            runtime_execution.execution_identifier.as_str(),
            runtime_execution.flow_id,
            runtime_execution.node_execution_results.len()
        );

        normalize_execution_result(&mut runtime_execution);

        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            ExecutionRequest {
                execution_result: Some(runtime_execution),
            },
        );

        match self.client.update(request).await {
            Ok(response) => {
                log::info!(
                    "Transmitted Execution Result (success: {})",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update RuntimeExecution: {:?}", err);
            }
        }
    }
}

fn normalize_execution_result(result: &mut ExecutionResult) {
    match &mut result.input {
        Some(input) => normalize_value(input),
        None => {
            result.input = Some(null_value());
        }
    }

    for node_result in &mut result.node_execution_results {
        normalize_node_execution_result(node_result);
    }

    match &mut result.result {
        Some(execution_result::Result::Success(value)) => normalize_value(value),
        Some(execution_result::Result::Error(error)) => {
            if let Some(details) = &mut error.details {
                for value in details.fields.values_mut() {
                    normalize_value(value);
                }
            }
        }
        None => {
            result.result = Some(execution_result::Result::Success(null_value()));
        }
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
        Some(node_execution_result::Result::Success(value)) => normalize_value(value),
        Some(node_execution_result::Result::Error(error)) => {
            if let Some(details) = &mut error.details {
                for value in details.fields.values_mut() {
                    normalize_value(value);
                }
            }
        }
        None => {
            result.result = Some(node_execution_result::Result::Success(null_value()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use tucana::shared::{
        Error, ListValue, NodeParameterNodeExecutionResult, NumberValue, Struct,
        node_execution_result,
    };

    #[test]
    fn normalize_execution_result_fills_missing_results_with_null_success() {
        let mut result = ExecutionResult {
            execution_identifier: "execution-id".to_string(),
            flow_id: 1,
            started_at: 1,
            finished_at: 2,
            input: None,
            node_execution_results: vec![NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: vec![NodeParameterNodeExecutionResult { value: None }],
                id: Some(node_execution_result::Id::NodeId(10)),
                result: None,
            }],
            result: None,
        };

        normalize_execution_result(&mut result);

        assert!(matches!(
            result.input.as_ref(),
            Some(Value {
                kind: Some(Kind::NullValue(_))
            })
        ));
        assert!(matches!(
            result.result.as_ref(),
            Some(execution_result::Result::Success(Value {
                kind: Some(Kind::NullValue(_))
            }))
        ));
        assert!(matches!(
            result.node_execution_results[0].result.as_ref(),
            Some(node_execution_result::Result::Success(Value {
                kind: Some(Kind::NullValue(_))
            }))
        ));
        assert!(matches!(
            result.node_execution_results[0].parameter_results[0]
                .value
                .as_ref(),
            Some(Value {
                kind: Some(Kind::NullValue(_))
            })
        ));
    }

    #[test]
    fn normalize_execution_result_recurses_into_child_values() {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "list".to_string(),
            Value {
                kind: Some(Kind::ListValue(ListValue {
                    values: vec![
                        Value { kind: None },
                        Value {
                            kind: Some(Kind::NumberValue(NumberValue { number: None })),
                        },
                    ],
                })),
            },
        );
        let mut result = ExecutionResult {
            execution_identifier: "execution-id".to_string(),
            flow_id: 1,
            started_at: 1,
            finished_at: 2,
            input: Some(Value {
                kind: Some(Kind::StructValue(Struct { fields })),
            }),
            node_execution_results: vec![NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: vec![NodeParameterNodeExecutionResult {
                    value: Some(Value { kind: None }),
                }],
                id: Some(node_execution_result::Id::NodeId(10)),
                result: Some(node_execution_result::Result::Error(Error {
                    details: Some(Struct {
                        fields: std::collections::HashMap::from([(
                            "empty".to_string(),
                            Value { kind: None },
                        )]),
                    }),
                    ..Default::default()
                })),
            }],
            result: Some(execution_result::Result::Success(Value { kind: None })),
        };

        normalize_execution_result(&mut result);

        assert!(matches!(
            result.result.as_ref(),
            Some(execution_result::Result::Success(Value {
                kind: Some(Kind::NullValue(_))
            }))
        ));
        assert!(matches!(
            result.node_execution_results[0].parameter_results[0]
                .value
                .as_ref(),
            Some(Value {
                kind: Some(Kind::NullValue(_))
            })
        ));
        let Some(Value {
            kind: Some(Kind::StructValue(input)),
        }) = result.input.as_ref()
        else {
            panic!("expected normalized struct input");
        };
        let Some(Value {
            kind: Some(Kind::ListValue(list)),
        }) = input.fields.get("list")
        else {
            panic!("expected normalized list field");
        };
        assert!(matches!(
            list.values[0].kind.as_ref(),
            Some(Kind::NullValue(_))
        ));
        assert!(matches!(
            list.values[1].kind.as_ref(),
            Some(Kind::NullValue(_))
        ));
        let Some(node_execution_result::Result::Error(error)) =
            result.node_execution_results[0].result.as_ref()
        else {
            panic!("expected node error result");
        };
        assert!(matches!(
            error
                .details
                .as_ref()
                .and_then(|details| details.fields.get("empty"))
                .and_then(|value| value.kind.as_ref()),
            Some(Kind::NullValue(_))
        ));
    }
}
