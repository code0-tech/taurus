//! Delegates remote node execution to another service over NATS request/reply:
//! publishes an `ActionExecutionRequest` on `action.<target_service>.<execution_id>`
//! with a fresh reply inbox, then waits for the matching `ActionExecutionResponse`.
//!
//! For an ordinary call (no sub-flow parameters minted) the wait is bounded
//! by one flat `execution_result_timeout` from the moment the request is
//! sent, same as always. A call that *did* mint at least one sub-flow
//! reference (`RemoteExecution::sub_flow_activity` is `Some`) instead waits
//! on a *renewable* idle timeout: every time the `sub_flow_execution.*`
//! subscriber (`taurus/src/app/worker.rs`) looks up and runs one of this
//! call's minted sub-flows, it notifies the shared `Notify`, which resets
//! the deadline here. The call only fails if the reply never arrives *and*
//! nothing on the sub-flow channel happens for a full `execution_result_timeout`
//! window -- so a call meant to stay open for hours survives as long as the
//! action keeps driving sub-flow traffic (or eventually replies).

use std::sync::Arc;
use std::time::Duration;

use async_nats::{Client, Message, Subscriber};
use futures_lite::StreamExt;
use prost::Message as _;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tokio::sync::Notify;
use tonic::async_trait;
use tucana::aquila::ActionExecutionResponse;
use tucana::shared::NodeExecutionResult;

// `Client` is a cheap Arc-backed handle, so this is cheap to clone per
// concurrently-executing flow.
#[derive(Clone)]
pub struct NATSRemoteRuntime {
    client: Client,
    execution_result_timeout: Duration,
}

impl NATSRemoteRuntime {
    pub fn with_execution_result_timeout(
        client: Client,
        execution_result_timeout: Duration,
    ) -> Self {
        NATSRemoteRuntime {
            client,
            execution_result_timeout,
        }
    }
}

#[async_trait]
impl RemoteRuntime for NATSRemoteRuntime {
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        let topic = format!(
            "action.{}.{}",
            execution.target_service, execution.request.execution_identifier
        );
        let payload = execution.request.encode_to_vec();

        log::info!("Request Remote Runtime Execution with topic: : {}", topic);
        let inbox = self.client.new_inbox();
        let mut sub = match self.client.subscribe(inbox.clone()).await {
            Ok(sub) => sub,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to subscribe to NATS reply inbox: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };
        if let Err(err) = self
            .client
            .publish_with_reply(topic, inbox, payload.into())
            .await
        {
            log::error!(
                "RemoteRuntimeException: failed to publish NATS request: {}",
                err
            );
            return Err(RuntimeError::new(
                "T-PROV-000001",
                "RemoteRuntimeException",
                "Failed to receive any response messages from a remote runtime.",
            ));
        }
        match tokio::time::timeout(self.execution_result_timeout, self.client.flush()).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                log::error!(
                    "RemoteRuntimeException: failed to flush NATS request: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to flush NATS request before timeout: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        }

        let message = match wait_for_reply(
            &mut sub,
            self.execution_result_timeout,
            execution.sub_flow_activity.as_ref(),
        )
        .await
        {
            ReplyOutcome::Message(message) => message,
            ReplyOutcome::Closed => {
                log::error!("RemoteRuntimeException: NATS reply subscription closed");
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
            ReplyOutcome::TimedOut => {
                log::error!(
                    "RemoteRuntimeException: failed to receive NATS response before timeout"
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };

        let decode_result = ActionExecutionResponse::decode(message.payload);
        match decode_result {
            Ok(r) => match r.node_result {
                Some(res) => Ok(res),
                None => {
                    log::error!("RemoteRuntimeException: received execution result without a body");
                    Err(RuntimeError::new(
                        "T-PROV-000003",
                        "RemoteRuntimeException",
                        "Received empty action execution response",
                    ))
                }
            },
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to decode NATS message: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000002",
                    "RemoteRuntimeException",
                    "Failed to read Remote Response",
                ));
            }
        }
    }
}

enum ReplyOutcome {
    Message(Message),
    /// The reply subscription ended without ever producing a message.
    Closed,
    /// No reply arrived (and, for a sub-flow-activity call, no activity
    /// either) within the timeout window.
    TimedOut,
}

/// Waits for the next message on `sub`.
///
/// * `activity` is `None` for an ordinary call: waits with one flat
///   `timeout` from the moment this function is called, exactly like the
///   single `tokio::time::timeout(...)` this replaces.
/// * `activity` is `Some` for a call that minted at least one sub-flow
///   reference: the `timeout` window restarts every time `activity` is
///   notified (bumped by the `sub_flow_execution.*` subscriber on every
///   successful lookup+run for this call's parent id), so genuine sub-flow
///   traffic keeps the wait alive indefinitely -- it only fails on a window
///   with no reply *and* no sub-flow activity at all.
async fn wait_for_reply(
    sub: &mut Subscriber,
    timeout: Duration,
    activity: Option<&Arc<Notify>>,
) -> ReplyOutcome {
    let Some(activity) = activity else {
        return match tokio::time::timeout(timeout, sub.next()).await {
            Ok(Some(message)) => ReplyOutcome::Message(message),
            Ok(None) => ReplyOutcome::Closed,
            Err(_) => ReplyOutcome::TimedOut,
        };
    };

    loop {
        tokio::select! {
            message = sub.next() => {
                return match message {
                    Some(message) => ReplyOutcome::Message(message),
                    None => ReplyOutcome::Closed,
                };
            }
            // Activity resets the deadline: looping back around creates a
            // fresh `sleep` future below instead of reusing the elapsed one.
            _ = activity.notified() => continue,
            _ = tokio::time::sleep(timeout) => return ReplyOutcome::TimedOut,
        }
    }
}

/// These tests need a real NATS server (CI already runs one as a service on
/// `127.0.0.1:4222` -- see `.github/workflows/build-and-test.yml`); they are
/// `#[ignore]`d so `cargo test` stays hermetic by default. Run locally with
/// a NATS server up (`nats-server` or `docker run -p 4222:4222 nats:2`) via
/// `cargo test -p taurus-provider -- --ignored`, optionally overriding
/// `NATS_URL`.
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tucana::aquila::ActionExecutionRequest;
    use tucana::shared::{node_execution_result, value::Kind};

    /// Cheap process-unique id for test topics -- avoids colliding with any
    /// other concurrently-running test against the same NATS server, without
    /// pulling in a UUID dependency this crate doesn't otherwise need.
    fn unique_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        format!(
            "test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    async fn test_client() -> Client {
        let url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
        async_nats::connect(url)
            .await
            .expect("connect to local NATS test server")
    }

    fn build_execution(
        service: &str,
        execution_identifier: &str,
        sub_flow_activity: Option<Arc<Notify>>,
    ) -> RemoteExecution {
        RemoteExecution {
            target_service: service.to_string(),
            request: ActionExecutionRequest {
                execution_identifier: execution_identifier.to_string(),
                function_identifier: "test::fn".to_string(),
                parameters: Vec::new(),
                project_id: 1,
            },
            sub_flow_activity,
        }
    }

    /// Subscribes to `action.<service>.<execution_identifier>` and receives
    /// the request but never replies -- standing in for an action that's
    /// gone silent. Subscribing happens synchronously, *before* returning,
    /// so the caller can publish the request immediately afterwards without
    /// racing the spawned task for the subscription: a subscriber must
    /// already exist when the request is published, otherwise NATS core's
    /// "no responders" fast-path immediately synthesizes an empty reply,
    /// which would make the call fail instantly instead of actually waiting
    /// out the timeout under test.
    async fn spawn_silent_responder(client: Client, service: &str, execution_identifier: &str) {
        let topic = format!("action.{}.{}", service, execution_identifier);
        let mut sub = client
            .subscribe(topic)
            .await
            .expect("subscribe to request topic");
        tokio::spawn(async move {
            let _ = sub.next().await;
            // Deliberately never replies; keep the subscription alive for
            // the rest of the test so no further "no responders" notice
            // can be synthesized either.
            std::future::pending::<()>().await;
        });
    }

    /// Waits for the next request on `action.<service>.<execution_identifier>`
    /// and answers it with a canned success response after `delay`, standing
    /// in for a slow-but-alive action. See `spawn_silent_responder` for why
    /// subscribing happens synchronously before returning.
    async fn spawn_delayed_responder(
        client: Client,
        service: &str,
        execution_identifier: &str,
        delay: Duration,
    ) {
        let topic = format!("action.{}.{}", service, execution_identifier);
        let mut sub = client
            .subscribe(topic)
            .await
            .expect("subscribe to request topic");
        let execution_identifier = execution_identifier.to_string();
        tokio::spawn(async move {
            let message = sub.next().await.expect("request should arrive");
            let reply = message.reply.expect("request should carry a reply subject");
            tokio::time::sleep(delay).await;
            let response = ActionExecutionResponse {
                execution_identifier,
                node_result: Some(NodeExecutionResult {
                    started_at: 0,
                    finished_at: 0,
                    parameter_results: Vec::new(),
                    id: None,
                    result: Some(node_execution_result::Result::Success(
                        tucana::shared::Value {
                            kind: Some(Kind::BoolValue(true)),
                        },
                    )),
                }),
            };
            client
                .publish(reply, response.encode_to_vec().into())
                .await
                .expect("publish reply");
            client.flush().await.expect("flush reply");
        });
    }

    #[tokio::test]
    #[ignore = "requires a real NATS server, see module docs"]
    async fn ordinary_call_times_out_at_flat_deadline_when_no_activity() {
        let client = test_client().await;
        let runtime = NATSRemoteRuntime::with_execution_result_timeout(
            client.clone(),
            Duration::from_millis(200),
        );
        let execution_identifier = unique_id();
        spawn_silent_responder(client, "svc", &execution_identifier).await;
        let execution = build_execution("svc", &execution_identifier, None);

        let started = Instant::now();
        let result = runtime.execute_remote(execution).await;
        let elapsed = started.elapsed();

        assert!(
            result.is_err(),
            "expected a timeout failure, got {:?}",
            result
        );
        assert!(
            elapsed >= Duration::from_millis(200),
            "should not fail before the flat deadline, elapsed={:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_secs(2),
            "should fail at roughly the flat deadline, elapsed={:?}",
            elapsed
        );
    }

    #[tokio::test]
    #[ignore = "requires a real NATS server, see module docs"]
    async fn sub_flow_activity_keeps_call_alive_past_the_flat_deadline() {
        let client = test_client().await;
        let flat_timeout = Duration::from_millis(200);
        let runtime =
            NATSRemoteRuntime::with_execution_result_timeout(client.clone(), flat_timeout);
        let execution_identifier = unique_id();
        let activity = Arc::new(Notify::new());

        // The reply lands well after the flat deadline would have fired.
        let reply_delay = flat_timeout * 3;
        spawn_delayed_responder(client, "svc", &execution_identifier, reply_delay).await;

        // Simulate the `sub_flow_execution.*` subscriber bumping activity on
        // every sub-flow lookup+run, faster than the flat deadline, for
        // longer than the flat deadline alone would tolerate.
        let notifier_activity = Arc::clone(&activity);
        tokio::spawn(async move {
            for _ in 0..8 {
                tokio::time::sleep(flat_timeout / 3).await;
                notifier_activity.notify_one();
            }
        });

        let execution = build_execution("svc", &execution_identifier, Some(activity));
        let started = Instant::now();
        let result = runtime.execute_remote(execution).await;
        let elapsed = started.elapsed();

        assert!(
            result.is_ok(),
            "sub-flow activity should have kept the call alive, got {:?}",
            result
        );
        assert!(
            elapsed >= reply_delay,
            "should only resolve once the reply actually arrives, elapsed={:?}",
            elapsed
        );
    }

    #[tokio::test]
    #[ignore = "requires a real NATS server, see module docs"]
    async fn sub_flow_activity_silence_past_window_still_times_out() {
        let client = test_client().await;
        let runtime = NATSRemoteRuntime::with_execution_result_timeout(
            client.clone(),
            Duration::from_millis(200),
        );
        let execution_identifier = unique_id();
        spawn_silent_responder(client, "svc", &execution_identifier).await;
        // `Some` activity handle, but nothing ever notifies it -- must
        // behave the same as an ordinary call with no activity to track.
        let activity = Arc::new(Notify::new());
        let execution = build_execution("svc", &execution_identifier, Some(activity));

        let started = Instant::now();
        let result = runtime.execute_remote(execution).await;
        let elapsed = started.elapsed();

        assert!(
            result.is_err(),
            "expected a timeout failure, got {:?}",
            result
        );
        assert!(
            elapsed >= Duration::from_millis(200) && elapsed < Duration::from_secs(2),
            "should time out at roughly the idle window, elapsed={:?}",
            elapsed
        );
    }
}
