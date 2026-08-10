//! Remote runtime execution interface.
//!
//! Local runtime nodes can delegate execution to remote services through this
//! trait without coupling the core engine to a specific transport.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Notify;
use tucana::{aquila::ActionExecutionRequest, shared::NodeExecutionResult};

use crate::types::errors::runtime_error::RuntimeError;

pub struct RemoteExecution {
    /// Remote service identifier to route the call.
    pub target_service: String,
    /// Execution request payload expected by the remote runtime.
    pub request: ActionExecutionRequest,
    /// Set only when `request` carries at least one minted sub-flow
    /// reference. A `RemoteRuntime` implementation that supports the
    /// renewable idle-timeout keepalive (see `NATSRemoteRuntime`) should
    /// reset its wait deadline every time this is notified instead of
    /// enforcing a single flat deadline from call start. `None` means this
    /// call has no sub-flow traffic to wait on, so implementations must
    /// fall back to their ordinary flat timeout unchanged.
    pub sub_flow_activity: Option<Arc<Notify>>,
}

#[async_trait]
pub trait RemoteRuntime: Send + Sync {
    /// Execute a remote node invocation and return its resulting value.
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError>;
}
