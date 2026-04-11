use std::collections::HashMap;

use async_nats::Client;
use async_trait::async_trait;
use clap::{Parser, arg, command};
use prost::Message;
use taurus_core::context::{context::Context, executor::Executor, registry::FunctionStore};
use taurus_core::runtime::{error::RuntimeError, remote::RemoteRuntime};
use tests_core::Case;
use tucana::shared::helper::value::to_json_value;
use tucana::shared::{NodeFunction, helper::value::from_json_value};
use tucana::{
    aquila::{ExecutionRequest, ExecutionResult},
    shared::Value,
};

pub struct RemoteNatsClient {
    client: Client,
}

impl RemoteNatsClient {
    pub fn new(client: Client) -> Self {
        RemoteNatsClient { client }
    }
}

#[async_trait]
impl RemoteRuntime for RemoteNatsClient {
    async fn execute_remote(
        &self,
        remote_name: String,
        request: ExecutionRequest,
    ) -> Result<Value, RuntimeError> {
        let topic = format!("action.{}.{}", remote_name, request.execution_identifier);
        let payload = request.encode_to_vec();
        let res = self.client.request(topic.clone(), payload.into()).await;
        log::info!("Publishing to topic: {}", topic);
        let message = match res {
            Ok(r) => r,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeExeption: failed to handle NATS message: {}",
                    err
                );
                return Err(RuntimeError::simple_str(
                    "RemoteRuntimeExeption",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };

        let decode_result = ExecutionResult::decode(message.payload);
        let execution_result = match decode_result {
            Ok(r) => r,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeExeption: failed to decode NATS message: {}",
                    err
                );
                return Err(RuntimeError::simple_str(
                    "RemoteRuntimeExeption",
                    "Failed to read Remote Response",
                ));
            }
        };

        match execution_result.result {
            Some(result) => match result {
                tucana::aquila::execution_result::Result::Success(value) => Ok(value),
                tucana::aquila::execution_result::Result::Error(err) => {
                    let name = err.code.to_string();
                    let description = match err.description {
                        Some(string) => string,
                        None => "Unknown Error".to_string(),
                    };
                    let error = RuntimeError::new(name, description, None);
                    Err(error)
                }
            },
            None => Err(RuntimeError::simple_str(
                "RemoteRuntimeExeption",
                "Result of Remote Response was empty.",
            )),
        }
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

    let store = FunctionStore::default();

    let node_functions: HashMap<i64, NodeFunction> = case
        .clone()
        .flow
        .node_functions
        .into_iter()
        .map(|node| (node.database_id, node))
        .collect();

    let mut context = match case.inputs.get(index as usize) {
        Some(inp) => match inp.input.clone() {
            Some(json_input) => Context::new(from_json_value(json_input)),
            None => Context::default(),
        },
        None => Context::default(),
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
    let remote = RemoteNatsClient::new(client);
    let result = Executor::new(&store, node_functions.clone())
        .with_remote_runtime(&remote)
        .execute(case.flow.starting_node_id, &mut context, true);

    match result {
        taurus_core::context::signal::Signal::Success(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        taurus_core::context::signal::Signal::Return(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        taurus_core::context::signal::Signal::Respond(value) => {
            let json = to_json_value(value);
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty);
        }
        taurus_core::context::signal::Signal::Stop => println!("Received Stop signal"),
        taurus_core::context::signal::Signal::Failure(runtime_error) => {
            println!("RuntimeError: {:?}", runtime_error);
        }
    }
}
