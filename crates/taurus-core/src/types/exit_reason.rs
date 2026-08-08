//! Final execution exit reason for runtime calls.
//!
//! This captures why an execution boundary (flow run or nested call frame)
//! ended. It is intentionally payload-free and stable for logging, metrics,
//! and control decisions.

use std::fmt::{Display, Formatter};

/// Why execution ended at an execution boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// Execution reached normal completion with a success value.
    Success,
    /// Execution ended with a runtime failure.
    Failure,
    /// Execution ended due to an explicit `return`.
    Return,
    /// Execution ended due to an explicit `stop`.
    Stop,
}

impl ExitReason {
    /// True when execution ended in an error state.
    pub const fn is_failure(self) -> bool {
        matches!(self, ExitReason::Failure)
    }
}

impl Display for ExitReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ExitReason::Success => "success",
            ExitReason::Failure => "failure",
            ExitReason::Return => "return",
            ExitReason::Stop => "stop",
        };

        write!(f, "{label}")
    }
}
