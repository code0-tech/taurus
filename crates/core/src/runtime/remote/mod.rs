use async_trait::async_trait;
use tucana::{aquila::ExecutionRequest, shared::Value};

use crate::runtime::error::RuntimeError;

#[async_trait]
pub trait RemoteRuntime {
    async fn execute_remote(
        &self,
        remote_name: String,
        request: ExecutionRequest,
    ) -> Result<Value, RuntimeError>;
}
