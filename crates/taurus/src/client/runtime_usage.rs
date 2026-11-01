//! Group-commits per-execution runtime usage (flow id + duration) into the
//! JetStream work-queue stream Aquila drains and forwards to Sagittarius.
//!
//! Recording must never slow a flow execution down, so
//! [`RuntimeUsageRecorder::record`] only does a non-blocking `try_send` onto
//! a bounded channel. A single background task ([`spawn`]) collects the
//! entries and publishes them as one `RuntimeUsageRequest` per batch -- the
//! same message Aquila forwards to Sagittarius, so it can merge batches
//! without re-encoding individual entries. A batch is flushed as soon as
//! `batch_size` entries are buffered or `flush_interval` elapses, whichever
//! comes first. While NATS is unavailable, unpublished entries stay buffered
//! (bounded by `buffer_capacity`) and are retried on the next flush; beyond
//! that, new usages are dropped rather than applying backpressure to
//! executions.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_nats::jetstream::{self, stream::RetentionPolicy};
use prost::Message;
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio::task::JoinHandle;
use tucana::sagittarius_gateway::RuntimeUsageRequest;
use tucana::shared::RuntimeUsage;

use crate::telemetry::errors;

pub struct RuntimeUsageSettings {
    /// JetStream stream (and subject) usages are published to; must match
    /// the stream Aquila consumes from.
    pub stream: String,
    /// Maximum number of usages published as one NATS message.
    pub batch_size: usize,
    /// Maximum time a recorded usage waits before its batch is flushed.
    pub flush_interval: Duration,
    /// Maximum number of usages buffered in memory (channel plus entries
    /// awaiting a successful publish).
    pub buffer_capacity: usize,
}

/// Cheap, cloneable handle the worker uses to record one usage per flow
/// execution.
#[derive(Clone)]
pub struct RuntimeUsageRecorder {
    sender: mpsc::Sender<RuntimeUsage>,
    dropped: Arc<AtomicU64>,
}

impl RuntimeUsageRecorder {
    /// Queues one execution's usage for the next group commit. Never blocks:
    /// if the buffer is full the usage is dropped and counted instead.
    pub fn record(&self, flow_id: i64, duration_micros: i64) {
        let usage = RuntimeUsage {
            flow_id,
            duration: duration_micros.max(0),
        };

        match self.sender.try_send(usage) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                // Log the first drop and then every 1000th, so a sustained
                // outage stays visible without flooding the log.
                if dropped == 1 || dropped.is_multiple_of(1000) {
                    log::warn!(
                        "Runtime usage buffer is full; dropped {} usage(s) so far (flow_id={})",
                        dropped,
                        flow_id
                    );
                }
            }
            Err(TrySendError::Closed(_)) => {
                log::debug!(
                    "Runtime usage publisher has stopped; dropping usage for flow {}",
                    flow_id
                );
            }
        }
    }
}

/// Starts the background publisher and returns the recorder handle plus the
/// publisher's task. The task flushes everything still buffered and exits
/// once every [`RuntimeUsageRecorder`] clone has been dropped.
pub fn spawn(
    client: async_nats::Client,
    settings: RuntimeUsageSettings,
) -> (RuntimeUsageRecorder, JoinHandle<()>) {
    let batch_size = settings.batch_size.max(1);
    let buffer_capacity = settings.buffer_capacity.max(batch_size);
    let (sender, receiver) = mpsc::channel(buffer_capacity);

    let publisher = RuntimeUsagePublisher {
        jetstream: jetstream::new(client),
        stream: settings.stream,
        batch_size,
        buffer_capacity,
        stream_ready: false,
        backing_off: false,
        pending: Vec::with_capacity(batch_size),
    };

    let handle = tokio::spawn(publisher.run(receiver, settings.flush_interval));

    (
        RuntimeUsageRecorder {
            sender,
            dropped: Arc::new(AtomicU64::new(0)),
        },
        handle,
    )
}

struct RuntimeUsagePublisher {
    jetstream: jetstream::Context,
    stream: String,
    batch_size: usize,
    buffer_capacity: usize,
    stream_ready: bool,
    /// Set after a failed publish: further publishes wait for the next
    /// flush tick instead of retrying on every received usage.
    backing_off: bool,
    /// Usages received but not yet acknowledged by JetStream.
    pending: Vec<RuntimeUsage>,
}

impl RuntimeUsagePublisher {
    async fn run(mut self, mut receiver: mpsc::Receiver<RuntimeUsage>, flush_interval: Duration) {
        let mut ticker = tokio::time::interval(flush_interval.max(Duration::from_millis(1)));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut received = Vec::with_capacity(self.batch_size);

        log::info!(
            "Runtime usage publisher started (stream={}, batch_size={}, flush_interval={}ms)",
            self.stream,
            self.batch_size,
            flush_interval.as_millis()
        );

        loop {
            tokio::select! {
                count = receiver.recv_many(&mut received, self.batch_size) => {
                    // `recv_many` only returns 0 once the channel is closed
                    // and drained, i.e. every recorder has been dropped.
                    if count == 0 {
                        break;
                    }
                    self.pending.append(&mut received);
                    if self.backing_off {
                        self.enforce_buffer_capacity();
                    } else if self.pending.len() >= self.batch_size {
                        self.flush(false).await;
                    }
                }
                _ = ticker.tick() => {
                    if !self.pending.is_empty() {
                        self.flush(true).await;
                    }
                }
            }
        }

        self.flush(true).await;
        if !self.pending.is_empty() {
            log::warn!(
                "Runtime usage publisher stopped with {} unpublished usage(s)",
                self.pending.len()
            );
        }
        log::info!("Runtime usage publisher stopped");
    }

    /// Publishes buffered usages in `batch_size` chunks. Without
    /// `include_partial`, a trailing chunk smaller than `batch_size` stays
    /// buffered for the next flush. Stops at the first failed publish and
    /// keeps the remainder (bounded by `buffer_capacity`) for a later retry.
    async fn flush(&mut self, include_partial: bool) {
        while !self.pending.is_empty() && (include_partial || self.pending.len() >= self.batch_size)
        {
            let chunk_len = self.pending.len().min(self.batch_size);
            let request = RuntimeUsageRequest {
                runtime_usage: self.pending[..chunk_len].to_vec(),
            };

            if let Err(err) = self.publish(request).await {
                log::warn!(
                    "Failed to publish {} runtime usage(s); keeping them for retry: {}",
                    chunk_len,
                    err
                );
                self.stream_ready = false;
                self.backing_off = true;
                self.enforce_buffer_capacity();
                return;
            }

            self.backing_off = false;
            self.pending.drain(..chunk_len);
            log::debug!("Published {} runtime usage(s)", chunk_len);
        }
    }

    async fn publish(&mut self, request: RuntimeUsageRequest) -> Result<(), String> {
        if !self.stream_ready {
            ensure_stream(&self.jetstream, &self.stream)
                .await
                .map_err(|err| {
                    errors::record(
                        "messaging",
                        "runtime_usage.stream.ensure",
                        err.as_ref(),
                        format!("stream={}", self.stream),
                    );
                    err.to_string()
                })?;
            self.stream_ready = true;
        }

        let payload = request.encode_to_vec();
        let ack = self
            .jetstream
            .publish(self.stream.clone(), payload.into())
            .await
            .map_err(|err| {
                errors::record(
                    "messaging",
                    "runtime_usage.publish",
                    &err,
                    format!("stream={}", self.stream),
                );
                err.to_string()
            })?;

        ack.await.map_err(|err| {
            errors::record(
                "messaging",
                "runtime_usage.publish.ack",
                &err,
                format!("stream={}", self.stream),
            );
            err.to_string()
        })?;

        Ok(())
    }

    /// Drops the oldest unpublished usages once the retry buffer exceeds its
    /// capacity, so a long NATS outage can't grow memory without bound.
    fn enforce_buffer_capacity(&mut self) {
        if self.pending.len() <= self.buffer_capacity {
            return;
        }

        let overflow = self.pending.len() - self.buffer_capacity;
        self.pending.drain(..overflow);
        log::warn!(
            "Runtime usage retry buffer exceeded {} entries; dropped {} oldest usage(s)",
            self.buffer_capacity,
            overflow
        );
    }
}

/// Creates the work-queue stream if it doesn't exist yet. Aquila creates the
/// same stream on its side, so whichever service starts first owns its
/// creation; the configuration must stay identical in both.
async fn ensure_stream(
    jetstream: &jetstream::Context,
    stream: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: stream.to_string(),
            subjects: vec![stream.to_string()],
            retention: RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorder_with_capacity(
        capacity: usize,
    ) -> (RuntimeUsageRecorder, mpsc::Receiver<RuntimeUsage>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (
            RuntimeUsageRecorder {
                sender,
                dropped: Arc::new(AtomicU64::new(0)),
            },
            receiver,
        )
    }

    #[test]
    fn record_queues_usage_with_duration() {
        let (recorder, mut receiver) = recorder_with_capacity(4);

        recorder.record(42, 1_500);

        let usage = receiver.try_recv().expect("usage should be queued");
        assert_eq!(usage.flow_id, 42);
        assert_eq!(usage.duration, 1_500);
    }

    #[test]
    fn record_clamps_negative_duration_to_zero() {
        let (recorder, mut receiver) = recorder_with_capacity(4);

        recorder.record(1, -5);

        assert_eq!(
            receiver
                .try_recv()
                .expect("usage should be queued")
                .duration,
            0
        );
    }

    #[test]
    fn record_drops_usage_instead_of_blocking_when_buffer_is_full() {
        let (recorder, mut receiver) = recorder_with_capacity(1);

        recorder.record(1, 10);
        recorder.record(2, 20);

        assert_eq!(recorder.dropped.load(Ordering::Relaxed), 1);
        assert_eq!(
            receiver
                .try_recv()
                .expect("first usage should be queued")
                .flow_id,
            1
        );
        assert!(receiver.try_recv().is_err());
    }
}
