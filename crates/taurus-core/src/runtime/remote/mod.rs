//! Remote runtime execution interface.
//!
//! Local runtime nodes can delegate execution to remote services through this
//! trait without coupling the core engine to a specific transport.

use async_trait::async_trait;
use tucana::{aquila::ActionExecutionRequest, shared::Value};

use crate::types::errors::runtime_error::RuntimeError;

pub struct RemoteExecution {
    /// Remote service identifier to route the call.
    pub target_service: String,
    /// Execution request payload expected by the remote runtime.
    pub request: ActionExecutionRequest,

}

#[async_trait]
pub trait RemoteRuntime {
    /// Execute a remote node invocation and return its resulting value.
    async fn execute_remote(&self, execution: RemoteExecution) -> Result<Value, RuntimeError>;
}
