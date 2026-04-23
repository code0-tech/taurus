//! Control-flow and result signals exchanged between runtime handlers and executor.
//!
//! This module defines the canonical signal vocabulary for Taurus runtime execution.

use std::fmt::{Display, Formatter};

use tucana::shared::Value;

use crate::types::errors::runtime_error::RuntimeError;
use crate::types::exit_reason::ExitReason;

/// Runtime control signal emitted by function handlers and consumed by the executor.
///
/// These signals model both value production and control-flow decisions.
/// The executor interprets each variant as follows:
///
/// - [`Signal::Success`]: normal node completion; execution continues through `next_node_id`.
/// - [`Signal::Failure`]: terminal error; current flow execution stops.
/// - [`Signal::Return`]: exits only the current call context. When returned from a lazily
///   executed child flow, the parent receives it as a successful value.
/// - [`Signal::Respond`]: out-of-band emission used for streaming replies; executor emits the
///   value via a configured callback and then continues the flow.
/// - [`Signal::Stop`]: explicit hard stop; execution ends immediately.
#[derive(Debug, Clone)]
pub enum Signal {
    /// Node execution completed successfully with a value.
    Success(Value),
    /// Node execution failed with a runtime error.
    Failure(RuntimeError),
    /// Return from the current call frame with a value.
    Return(Value),
    /// Emit an intermediate response value without terminating execution.
    Respond(Value),
    /// Stop execution immediately.
    Stop,
}

impl Signal {
    /// Return the terminal exit reason represented by this signal.
    pub const fn exit_reason(&self) -> ExitReason {
        match self {
            Signal::Success(_) => ExitReason::Success,
            Signal::Failure(_) => ExitReason::Failure,
            Signal::Return(_) => ExitReason::Return,
            Signal::Respond(_) => ExitReason::Respond,
            Signal::Stop => ExitReason::Stop,
        }
    }

    /// True when the signal ends the current call-frame execution loop.
    pub const fn is_terminal_in_frame(&self) -> bool {
        matches!(self, Signal::Failure(_) | Signal::Return(_) | Signal::Stop)
    }

    /// Borrow the value payload for value-carrying signals.
    pub const fn value(&self) -> Option<&Value> {
        match self {
            Signal::Success(v) | Signal::Return(v) | Signal::Respond(v) => Some(v),
            Signal::Failure(_) | Signal::Stop => None,
        }
    }

    /// Borrow the error payload when this is a failure signal.
    pub const fn error(&self) -> Option<&RuntimeError> {
        match self {
            Signal::Failure(err) => Some(err),
            Signal::Success(_) | Signal::Return(_) | Signal::Respond(_) | Signal::Stop => None,
        }
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Success(_) => write!(f, "Signal(success)"),
            Signal::Failure(err) => {
                write!(
                    f,
                    "Signal(failure, code={}, category={}, message={})",
                    err.code, err.category, err.message
                )
            }
            Signal::Return(_) => write!(f, "Signal(return)"),
            Signal::Respond(_) => write!(f, "Signal(respond)"),
            Signal::Stop => write!(f, "Signal(stop)"),
        }
    }
}

/// Partial equality by signal kind.
///
/// Payload values are intentionally ignored to keep tests focused on control-flow shape.
impl PartialEq for Signal {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Signal::Success(_), Signal::Success(_))
                | (Signal::Failure(_), Signal::Failure(_))
                | (Signal::Return(_), Signal::Return(_))
                | (Signal::Stop, Signal::Stop)
                | (Signal::Respond(_), Signal::Respond(_))
        )
    }
}
