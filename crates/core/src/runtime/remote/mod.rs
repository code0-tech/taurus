use std::future::Future;
use std::pin::Pin;
use tucana::aquila::{ActionRuntimeError, ExecutionRequest, ExecutionResult};

pub trait RemoteRuntime {
    fn supports(&self, function_identifier: &str) -> bool;

    fn execute_remote(
        &self,
        request: ExecutionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecutionResult, ActionRuntimeError>> + Send + '_>>;
}
