//! Observability wiring. `Telemetry`/`TelemetrySettings`/`errors` are
//! re-exported from `code0_flow`; [`metrics`] adds the flow/function
//! execution counters and histograms specific to Taurus.

pub mod metrics;

pub use code0_flow::flow_telemetry::{OpenTelemetry, Telemetry, TelemetrySettings, errors};
