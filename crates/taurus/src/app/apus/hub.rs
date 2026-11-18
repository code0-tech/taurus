//! The Apus v1 server (data plane): authenticates direct action connections,
//! multiplexes calls over them and serves the executions, sub-flow invocations
//! and streams the action starts. Routing (which Taurus instance serves which
//! project) is announced with `ROUTES`; configuration and flows stay in Aquila.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use apus_protocol::message::{self, Route, Routes, reset, status};
use apus_protocol::stream::{DataOutcome, Incoming, Outgoing, Side, StreamResult};
use apus_protocol::{
    ErrorInfo, FrameBuilder, Hello, PROTOCOL_VERSION, Reader, StreamOut, TraceContext, Welcome,
    cancel, feature, kind, limits, read_frame_limited, reject, scaling, tls,
};
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use taurus_core::value::raw_value_result;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, Semaphore, mpsc, oneshot};
use tucana::aquila::ActionExecutionRequest;
use tucana::shared::{NodeValueExecutionResult, Value, node_value_execution_result};

use super::convert;
use super::exec::{ExecContext, ExecFailure};

type BoxRead = Box<dyn AsyncRead + Send + Unpin>;
type BoxWrite = Box<dyn AsyncWrite + Send + Unpin>;
type StreamReceivers = Vec<(u32, oneshot::Receiver<StreamResult>)>;

/// Largest `HELLO` accepted.
const MAX_HELLO: usize = 64 * 1024;
/// Delay before answering a failed `HELLO` (slows down guessing).
const REJECT_DELAY: Duration = Duration::from_millis(50);
/// How long an action may finish in-flight work after its client certificate expired.
const CERTIFICATE_GRACE: Duration = Duration::from_secs(30);

/// Reply to a `CALL`: the action's return value or its reported error, with the
/// action's own timestamps (as in `NodeExecutionResult`).
pub enum CallReply {
    Ok {
        /// A plain value, or a function the action returned (tucana 0.0.83).
        value: NodeValueExecutionResult,
        started_at: i64,
        finished_at: i64,
    },
    Err {
        error: ErrorInfo,
        started_at: i64,
        finished_at: i64,
    },
}

/// An action-initiated request in flight: abort handle for `CANCEL`, and the
/// deadline the sweeper enforces (no timer per request).
struct RunningRequest {
    abort: tokio::task::AbortHandle,
    deadline: Option<Instant>,
    error_kind: u8,
}

/// Map split into independently locked shards: the reader task inserts, the
/// request tasks remove from other worker threads.
struct ShardedMap<V> {
    shards: Vec<Mutex<HashMap<u32, V>>>,
}

impl<V> ShardedMap<V> {
    fn new(shards: usize) -> Self {
        Self {
            shards: (0..shards).map(|_| Mutex::new(HashMap::new())).collect(),
        }
    }

    fn shard(&self, key: u32) -> std::sync::MutexGuard<'_, HashMap<u32, V>> {
        self.shards[key as usize % self.shards.len()]
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn contains(&self, key: u32) -> bool {
        self.shard(key).contains_key(&key)
    }

    fn remove(&self, key: u32) -> Option<V> {
        self.shard(key).remove(&key)
    }

    /// Removes and returns every entry for which `expired` is true.
    fn drain_where(&self, mut expired: impl FnMut(&V) -> bool) -> Vec<(u32, V)> {
        let mut out = Vec::new();
        for shard in &self.shards {
            let mut map = shard.lock().unwrap_or_else(|e| e.into_inner());
            let keys: Vec<u32> = map
                .iter()
                .filter(|(_, v)| expired(v))
                .map(|(k, _)| *k)
                .collect();
            for key in keys {
                if let Some(v) = map.remove(&key) {
                    out.push((key, v));
                }
            }
        }
        out
    }
}

/// One live action connection.
pub struct Connection {
    pub action: String,
    functions: HashMap<String, u16>,
    tx: mpsc::UnboundedSender<Vec<u8>>,
    pending: Mutex<HashMap<u32, oneshot::Sender<CallReply>>>,
    next_id: AtomicU32,
    /// `HELLO.maxInFlight`: CALLs this action accepts concurrently.
    call_slots: Option<Arc<Semaphore>>,
    /// Projects of the client certificate; `None` = every project of the action.
    projects: Option<HashSet<i64>>,
    /// (index, count) for `split`-scaled connections.
    shard: Option<(u32, u32)>,
    /// The action's `STATUS`; CALLs are only routed while it is running.
    status: AtomicU8,
    draining: AtomicBool,
    incoming: Mutex<Incoming>,
    outgoing: Arc<Outgoing>,
    next_stream_id: AtomicU32,
    stream_threshold: usize,
    /// EXEC + SUBFLOW requests currently running for this connection.
    requests_in_flight: AtomicU32,
    max_in_flight: u32,
    /// `STATUS not ready` was sent because the connection hit `max_in_flight`.
    overloaded: AtomicBool,
    request_tasks: ShardedMap<RunningRequest>,
    rx_frames: AtomicU64,
}

impl Connection {
    /// Whether this connection may serve `project_id` (ticket and shard).
    fn serves(&self, project_id: i64) -> bool {
        if let Some(projects) = &self.projects
            && !projects.contains(&project_id)
        {
            return false;
        }
        match self.shard {
            Some((index, count)) => project_id.rem_euclid(count as i64) == index as i64,
            None => true,
        }
    }

    fn routable(&self) -> bool {
        self.status.load(Ordering::Relaxed) == status::RUNNING
            && !self.draining.load(Ordering::Relaxed)
    }

    /// Sends a `CALL` and waits for `RETURN` / `ERROR`. The wait is renewed
    /// whenever `activity` fires (the action is driving a sub-flow of this call).
    pub async fn call(
        &self,
        method: u16,
        request: &ActionExecutionRequest,
        timeout: Duration,
        activity: Option<&Arc<Notify>>,
        trace: Option<&TraceContext>,
    ) -> Result<CallReply, RuntimeError> {
        // Load protection: never more outstanding calls than the action accepts.
        let _permit = match &self.call_slots {
            Some(slots) => match slots.clone().try_acquire_owned() {
                Ok(permit) => Some(permit),
                Err(_) => {
                    match tokio::time::timeout(timeout, slots.clone().acquire_owned()).await {
                        Ok(Ok(permit)) => Some(permit),
                        _ => {
                            return Err(RuntimeError::new(
                                "T-APUS-000004",
                                "ApusActionOverloaded",
                                "action has no free call slot before the deadline",
                            ));
                        }
                    }
                }
            },
            None => None,
        };

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let deadline_ms = timeout.as_millis().min(u32::MAX as u128) as u32;
        let mut frame = FrameBuilder::traced(kind::CALL, id, 256, trace);
        let mut streams = StreamOut::new(self.stream_threshold, &self.next_stream_id);
        {
            let body = frame.body();
            body.u16(method);
            body.str(&request.execution_identifier);
            body.i64(request.project_id);
            body.u32(deadline_ms);
            body.u8(request.parameters.len() as u8);
            let mut out = Some(&mut streams);
            for parameter in &request.parameters {
                convert::write_param(body, parameter, &mut out);
            }
        }

        let (reply_tx, reply_rx) = oneshot::channel();
        self.lock_pending().insert(id, reply_tx);
        // Removes the entry however this call ends, also when the execution
        // around it is aborted (CANCEL): no pending entry outlives its call.
        struct Pending<'a>(&'a Connection, u32, bool);
        impl Drop for Pending<'_> {
            fn drop(&mut self) {
                if self.0.lock_pending().remove(&self.1).is_some() && self.2 {
                    let _ = self
                        .0
                        .tx
                        .send(message::encode_cancel(self.1, cancel::CALLER));
                }
            }
        }
        let mut pending = Pending(self, id, true);
        if self.tx.send(frame.finish()).is_err() {
            pending.2 = false;
            return Err(transport_error("connection to action is closed"));
        }
        self.send_streams(streams.pending);

        match wait_for_reply(reply_rx, timeout, activity).await {
            Some(Ok(reply)) => Ok(reply),
            Some(Err(_)) => {
                pending.2 = false;
                Err(transport_error("connection to action closed during call"))
            }
            None => {
                pending.2 = false;
                self.lock_pending().remove(&id);
                let _ = self.tx.send(message::encode_cancel(id, cancel::DEADLINE));
                Err(transport_error("action did not answer before the deadline"))
            }
        }
    }

    fn send_streams(&self, pending: Vec<(u32, Vec<u8>)>) {
        for (id, bytes) in pending {
            self.outgoing.send(self.tx.clone(), id, bytes);
        }
    }

    fn lock_pending(&self) -> std::sync::MutexGuard<'_, HashMap<u32, oneshot::Sender<CallReply>>> {
        self.pending.lock().unwrap_or_else(|err| err.into_inner())
    }

    fn lock_incoming(&self) -> std::sync::MutexGuard<'_, Incoming> {
        self.incoming.lock().unwrap_or_else(|err| err.into_inner())
    }

    fn complete(&self, id: u32, reply: CallReply) {
        if let Some(tx) = self.lock_pending().remove(&id) {
            let _ = tx.send(reply);
        }
    }

    /// Registers the streams of a received value (in frame order, before their data).
    /// On refusal every stream of the value is reset and the error returned.
    fn expect_streams(&self, refs: &[(u32, i64)]) -> Result<StreamReceivers, String> {
        let mut incoming = self.lock_incoming();
        let mut receivers = Vec::with_capacity(refs.len());
        for (id, length) in refs {
            match incoming.expect(*id, *length) {
                Ok(rx) => receivers.push((*id, rx)),
                Err(code) => {
                    for (registered, _) in &receivers {
                        incoming.fail(*registered, reset::CANCELLED);
                    }
                    drop(incoming);
                    for (other, _) in refs {
                        let _ = self.tx.send(message::encode_reset(*other, code));
                    }
                    return Err(format!("stream {id} refused (code {code})"));
                }
            }
        }
        Ok(receivers)
    }

    fn fail_all(&self) {
        self.lock_pending().clear();
        self.lock_incoming().clear();
        self.outgoing.clear();
        for (_, request) in self.request_tasks.drain_where(|_| true) {
            request.abort.abort();
        }
    }
}

async fn wait_for_reply(
    reply_rx: oneshot::Receiver<CallReply>,
    timeout: Duration,
    activity: Option<&Arc<Notify>>,
) -> Option<Result<CallReply, oneshot::error::RecvError>> {
    let Some(activity) = activity else {
        return tokio::time::timeout(timeout, reply_rx).await.ok();
    };
    tokio::pin!(reply_rx);
    loop {
        tokio::select! {
            reply = &mut reply_rx => return Some(reply),
            _ = activity.notified() => continue,
            _ = tokio::time::sleep(timeout) => return None,
        }
    }
}

/// Waits for the streams of a value and fills their placeholders.
async fn complete_streams(receivers: StreamReceivers, values: &mut [Value]) -> Result<(), String> {
    let mut contents = HashMap::with_capacity(receivers.len());
    for (id, rx) in receivers {
        match rx.await {
            Ok(Ok(bytes)) => {
                contents.insert(id, bytes);
            }
            Ok(Err(code)) => return Err(format!("stream {id} was reset (code {code})")),
            Err(_) => return Err(format!("stream {id} was dropped")),
        }
    }
    for value in values {
        convert::fill_streams(value, &mut contents)?;
    }
    Ok(())
}

fn transport_error(message: &str) -> RuntimeError {
    RuntimeError::new("T-APUS-000001", "ApusTransport", message)
}

/// The routing table and the projects this instance serves.
struct Routing {
    routes: Routes,
    /// `None`: no table configured, this instance serves every project.
    served: Option<Served>,
}

/// What this instance serves according to the routing table.
struct Served {
    /// Projects listed for this instance.
    listed: HashSet<i64>,
    /// This instance is the default route: it also serves every project no
    /// other instance lists.
    default: bool,
    /// Projects listed for other instances (only needed for the default route).
    elsewhere: HashSet<i64>,
}

impl Served {
    fn new(routes: &Routes, instance_id: &str) -> Self {
        let mut listed = HashSet::new();
        let mut elsewhere = HashSet::new();
        for route in &routes.routes {
            let target = if route.instance_id == instance_id {
                &mut listed
            } else {
                &mut elsewhere
            };
            target.extend(route.projects.iter().copied());
        }
        // The first instance with an empty list is the default route (as `Routes::instance_for`).
        let default = routes
            .routes
            .iter()
            .find(|r| r.projects.is_empty())
            .is_some_and(|r| r.instance_id == instance_id);
        Self {
            listed,
            default,
            elsewhere,
        }
    }

    fn contains(&self, project_id: i64) -> bool {
        self.listed.contains(&project_id) || (self.default && !self.elsewhere.contains(&project_id))
    }

    fn is_empty(&self) -> bool {
        self.listed.is_empty() && !self.default
    }
}

/// Registry of connected actions, keyed by action identifier.
pub struct ApusHub {
    actions: RwLock<HashMap<String, Vec<Arc<Connection>>>>,
    round_robin: AtomicUsize,
    shards: Mutex<HashMap<String, Vec<bool>>>,
    routing: RwLock<Routing>,
    connections: AtomicUsize,
}

impl ApusHub {
    pub fn new(instance_id: &str, advertise: &str) -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
            round_robin: AtomicUsize::new(0),
            shards: Mutex::new(HashMap::new()),
            routing: RwLock::new(Routing {
                // Without a table this instance is the default route (empty project list).
                routes: Routes {
                    version: 0,
                    routes: vec![Route {
                        instance_id: instance_id.to_string(),
                        endpoint: advertise.to_string(),
                        projects: Vec::new(),
                    }],
                },
                served: None,
            }),
            connections: AtomicUsize::new(0),
        }
    }

    /// Picks a running connection of `service` that implements `function` and serves `project_id`.
    pub fn pick(
        &self,
        service: &str,
        function: &str,
        project_id: i64,
    ) -> Option<(Arc<Connection>, u16)> {
        let actions = self.actions.read().unwrap_or_else(|err| err.into_inner());
        let connections = actions.get(service)?;
        if connections.is_empty() {
            return None;
        }
        let start = self.round_robin.fetch_add(1, Ordering::Relaxed);
        for offset in 0..connections.len() {
            let connection = &connections[(start + offset) % connections.len()];
            if !connection.routable() || !connection.serves(project_id) {
                continue;
            }
            if let Some(method) = connection.functions.get(function) {
                return Some((connection.clone(), *method));
            }
        }
        None
    }

    fn serves_project(&self, project_id: i64) -> bool {
        let routing = self.routing.read().unwrap_or_else(|e| e.into_inner());
        routing
            .served
            .as_ref()
            .is_none_or(|served| served.contains(project_id))
    }

    fn routes_frame(&self) -> Vec<u8> {
        self.routing
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .routes
            .encode()
    }

    /// Installs a new routing table and announces it to every connected action.
    /// When this instance serves no project any more, actions are asked to leave.
    pub fn set_routes(&self, routes: Routes, instance_id: &str) {
        let served = Served::new(&routes, instance_id);
        let empty = served.is_empty();
        let frame = routes.encode();
        log::info!(
            "Apus routes v{}: instance {} serves {} project(s){}",
            routes.version,
            instance_id,
            served.listed.len(),
            if served.default {
                " and is the default route"
            } else {
                ""
            }
        );
        *self.routing.write().unwrap_or_else(|e| e.into_inner()) = Routing {
            routes,
            served: Some(served),
        };
        let actions = self.actions.read().unwrap_or_else(|err| err.into_inner());
        for connection in actions.values().flatten() {
            let _ = connection.tx.send(frame.clone());
            if empty {
                connection.draining.store(true, Ordering::Relaxed);
                let _ = connection.tx.send(message::encode_goaway(
                    0,
                    "this instance serves no project any more",
                ));
            }
        }
    }

    /// Graceful drain on shutdown: no new requests, in-flight ones finish.
    pub fn goaway_all(&self, reason: &str) {
        let actions = self.actions.read().unwrap_or_else(|err| err.into_inner());
        for connection in actions.values().flatten() {
            connection.draining.store(true, Ordering::Relaxed);
            let _ = connection.tx.send(message::encode_status(status::STOPPED));
            let _ = connection.tx.send(message::encode_goaway(0, reason));
        }
    }

    fn register(&self, connection: Arc<Connection>, max_per_action: usize) -> bool {
        let mut actions = self.actions.write().unwrap_or_else(|err| err.into_inner());
        let list = actions.entry(connection.action.clone()).or_default();
        if list.len() >= max_per_action {
            return false;
        }
        list.push(connection);
        true
    }

    fn unregister(&self, connection: &Arc<Connection>) {
        let mut actions = self.actions.write().unwrap_or_else(|err| err.into_inner());
        if let Some(list) = actions.get_mut(&connection.action) {
            list.retain(|c| !Arc::ptr_eq(c, connection));
        }
    }

    fn claim_shard(&self, action: &str, replicas: u32) -> Option<u32> {
        let mut shards = self.shards.lock().unwrap_or_else(|e| e.into_inner());
        let claimed = shards
            .entry(action.to_string())
            .or_insert_with(|| vec![false; replicas as usize]);
        let index = claimed.iter().position(|taken| !taken)?;
        claimed[index] = true;
        Some(index as u32)
    }

    fn release_shard(&self, action: &str, index: u32) {
        let mut shards = self.shards.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(slot) = shards
            .get_mut(action)
            .and_then(|c| c.get_mut(index as usize))
        {
            *slot = false;
        }
    }
}

pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub instance_id: String,
    /// EXEC + SUBFLOW requests accepted concurrently per connection.
    pub max_in_flight: u32,
    pub heartbeat: Duration,
    pub max_frame: u32,
    pub stream_window: u32,
    pub stream_threshold: u32,
    /// Mutual TLS: the client certificate identifies the action. `None`: plain TCP,
    /// nobody is authenticated (development).
    pub tls: Option<tls::TlsAcceptor>,
    pub max_connections: usize,
    pub max_connections_per_action: usize,
    pub handshake_timeout: Duration,
}

pub async fn serve(settings: ServerSettings, hub: Arc<ApusHub>, exec: Arc<ExecContext>) {
    match TcpListener::bind((settings.host.as_str(), settings.port)).await {
        Ok(listener) => serve_on(listener, settings, hub, exec).await,
        Err(err) => log::error!(
            "Apus server failed to bind {}:{}: {}",
            settings.host,
            settings.port,
            err
        ),
    }
}

pub async fn serve_on(
    listener: TcpListener,
    settings: ServerSettings,
    hub: Arc<ApusHub>,
    exec: Arc<ExecContext>,
) {
    log::info!(
        "Apus v{} server on {} instance={} auth={}",
        PROTOCOL_VERSION,
        listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_default(),
        settings.instance_id,
        if settings.tls.is_some() {
            "client certificates"
        } else {
            "none (no TLS)"
        },
    );
    let settings = Arc::new(settings);
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                if hub.connections.fetch_add(1, Ordering::Relaxed) >= settings.max_connections {
                    hub.connections.fetch_sub(1, Ordering::Relaxed);
                    log::warn!("Apus: connection limit reached, refusing {}", peer);
                    continue;
                }
                let hub = hub.clone();
                let exec = exec.clone();
                let settings = settings.clone();
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(stream, hub.clone(), exec, settings).await {
                        log::warn!("Apus connection from {} ended: {}", peer, err);
                    }
                    hub.connections.fetch_sub(1, Ordering::Relaxed);
                });
            }
            Err(err) => log::warn!("Apus accept failed: {}", err),
        }
    }
}

async fn reject_and_close(
    writer: &mut BufWriter<BoxWrite>,
    code: u16,
    text: &str,
) -> std::io::Result<()> {
    tokio::time::sleep(REJECT_DELAY).await;
    writer
        .write_all(&message::encode_reject(code, text))
        .await?;
    writer.flush().await?;
    Err(invalid(&format!("rejected HELLO: {text}")))
}

async fn handle_connection(
    tcp: TcpStream,
    hub: Arc<ApusHub>,
    exec: Arc<ExecContext>,
    settings: Arc<ServerSettings>,
) -> std::io::Result<()> {
    tcp.set_nodelay(true)?;
    // With TLS the verified client certificate says who the action is.
    let (read_half, write_half, identity): (
        BoxRead,
        BoxWrite,
        Option<Result<tls::PeerIdentity, String>>,
    ) = match &settings.tls {
        Some(acceptor) => {
            let stream = tokio::time::timeout(settings.handshake_timeout, acceptor.accept(tcp))
                .await
                .map_err(|_| invalid("TLS handshake timed out"))??;
            let identity = match stream
                .get_ref()
                .1
                .peer_certificates()
                .and_then(|certs| certs.first())
            {
                Some(cert) => tls::peer_identity(cert).map_err(|err| err.to_string()),
                None => Err("no client certificate".to_string()),
            };
            let (r, w) = tokio::io::split(stream);
            (Box::new(r), Box::new(w), Some(identity))
        }
        None => {
            let (r, w) = tcp.into_split();
            (Box::new(r), Box::new(w), None)
        }
    };
    let mut reader = BufReader::with_capacity(64 * 1024, read_half);
    let mut writer = BufWriter::with_capacity(64 * 1024, write_half);

    // --- Handshake: HELLO (within the timeout, size-limited) -> WELCOME + ROUTES, or REJECT ---
    let first = match tokio::time::timeout(
        settings.handshake_timeout,
        read_frame_limited(&mut reader, MAX_HELLO),
    )
    .await
    {
        Ok(Ok(Some(frame))) => frame,
        Ok(Ok(None)) => return Ok(()),
        Ok(Err(err)) => return Err(err),
        Err(_) => return Err(invalid("no HELLO within the handshake timeout")),
    };
    if first.kind != kind::HELLO {
        return reject_and_close(
            &mut writer,
            reject::PROTOCOL_ERROR,
            "first frame must be HELLO",
        )
        .await;
    }
    let hello = match Hello::decode(&first.body) {
        Ok(hello) => hello,
        Err(err) => {
            return reject_and_close(&mut writer, reject::PROTOCOL_ERROR, &err.to_string()).await;
        }
    };
    if hello.version != PROTOCOL_VERSION {
        return reject_and_close(
            &mut writer,
            reject::UNSUPPORTED_VERSION,
            "unsupported protocol version",
        )
        .await;
    }
    let identity = match identity {
        Some(Ok(identity)) if identity.action == hello.action => Some(identity),
        Some(Ok(_)) => {
            return reject_and_close(
                &mut writer,
                reject::UNAUTHENTICATED,
                "the client certificate is for another action",
            )
            .await;
        }
        Some(Err(text)) => {
            return reject_and_close(&mut writer, reject::UNAUTHENTICATED, &text).await;
        }
        None => None,
    };
    let replicas = identity.as_ref().map_or(1, |identity| identity.replicas);
    let shard = if hello.scaling == scaling::SPLIT && replicas > 1 {
        match hub.claim_shard(&hello.action, replicas) {
            Some(index) => Some((index, replicas)),
            None => {
                return reject_and_close(
                    &mut writer,
                    reject::SHARD_EXHAUSTED,
                    "all shards are claimed",
                )
                .await;
            }
        }
    } else {
        None
    };
    let action = hello.action.clone();
    let functions: HashMap<String, u16> = hello
        .functions
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i as u16))
        .collect();

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let stream_window = settings
        .stream_window
        .min(hello.stream_window.max(limits::STREAM_CHUNK as u32));
    let stream_threshold = if hello.features & feature::STREAMS != 0 {
        settings.stream_threshold as usize
    } else {
        0
    };
    let connection = Arc::new(Connection {
        action: action.clone(),
        functions,
        tx: tx.clone(),
        pending: Mutex::new(HashMap::new()),
        next_id: AtomicU32::new(1),
        call_slots: (hello.max_in_flight > 0)
            .then(|| Arc::new(Semaphore::new(hello.max_in_flight as usize))),
        projects: identity
            .as_ref()
            .and_then(|identity| identity.projects.clone())
            .map(|p| p.into_iter().collect()),
        shard,
        status: AtomicU8::new(status::NOT_READY),
        draining: AtomicBool::new(false),
        incoming: Mutex::new(Incoming::new(Side::Action)),
        outgoing: Arc::new(Outgoing::new(stream_window)),
        next_stream_id: AtomicU32::new(Side::Server.first_stream_id()),
        stream_threshold,
        requests_in_flight: AtomicU32::new(0),
        max_in_flight: settings.max_in_flight,
        overloaded: AtomicBool::new(false),
        request_tasks: ShardedMap::new(64),
        rx_frames: AtomicU64::new(0),
    });
    if !hub.register(connection.clone(), settings.max_connections_per_action) {
        if let Some((index, _)) = shard {
            hub.release_shard(&action, index);
        }
        return reject_and_close(
            &mut writer,
            reject::TOO_MANY_CONNECTIONS,
            "too many connections for this action",
        )
        .await;
    }

    let welcome = Welcome {
        version: PROTOCOL_VERSION,
        session_id: uuid::Uuid::new_v4().to_string(),
        instance_id: settings.instance_id.clone(),
        max_in_flight: settings.max_in_flight,
        max_frame: settings.max_frame,
        stream_window,
        stream_threshold: stream_threshold as u32,
        features: hello.features & feature::ALL,
        heartbeat_ms: settings.heartbeat.as_millis() as u32,
    };
    writer.write_all(&welcome.encode()).await?;
    writer.write_all(&hub.routes_frame()).await?;
    writer.flush().await?;

    // Writer task: drains queued frames and flushes once per batch.
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if writer.write_all(&frame).await.is_err() {
                return;
            }
            while let Ok(more) = rx.try_recv() {
                if writer.write_all(&more).await.is_err() {
                    return;
                }
            }
            if writer.flush().await.is_err() {
                return;
            }
        }
    });

    log::info!(
        "Apus action connected action={} functions={} shard={:?} projects={:?}",
        action,
        connection.functions.len(),
        connection.shard,
        connection.projects
    );

    let read_task = tokio::spawn(read_loop(
        reader,
        connection.clone(),
        exec,
        settings.clone(),
        hub.clone(),
    ));
    let heartbeat_task = tokio::spawn(heartbeat(
        connection.clone(),
        settings.heartbeat,
        read_task.abort_handle(),
    ));
    let sweeper_task = tokio::spawn(sweep(connection.clone()));
    let expiry_task = identity.as_ref().map(|identity| {
        tokio::spawn(expire(
            connection.clone(),
            identity.not_after,
            read_task.abort_handle(),
        ))
    });
    let result = match read_task.await {
        Ok(result) => result,
        Err(err) if err.is_cancelled() => Err(invalid("peer missed its heartbeats")),
        Err(err) => Err(invalid(&err.to_string())),
    };
    if result.is_err() {
        // Protocol error or dead peer: tell it why, best effort.
        let _ = tx.send(message::encode_goaway(
            reject::PROTOCOL_ERROR,
            "protocol error",
        ));
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    hub.unregister(&connection);
    connection.fail_all();
    if let Some((index, _)) = connection.shard {
        hub.release_shard(&action, index);
    }
    heartbeat_task.abort();
    sweeper_task.abort();
    if let Some(task) = expiry_task {
        task.abort();
    }
    writer_task.abort();
    log::info!("Apus action disconnected action={}", action);
    result
}

/// Ends a connection when its client certificate expires: TLS checks the certificate
/// only during the handshake, but connections live for days. `GOAWAY` first, so the
/// action renews its certificate (through Aquila) and reconnects; closed after a grace.
async fn expire(connection: Arc<Connection>, not_after: i64, read_task: tokio::task::AbortHandle) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    tokio::time::sleep(Duration::from_secs(
        not_after.saturating_sub(now).max(0) as u64
    ))
    .await;
    log::info!(
        "Apus client certificate expired action={}",
        connection.action
    );
    connection.draining.store(true, Ordering::Relaxed);
    let _ = connection.tx.send(message::encode_goaway(
        reject::UNAUTHENTICATED,
        "client certificate expired",
    ));
    tokio::time::sleep(CERTIFICATE_GRACE).await;
    read_task.abort();
}

/// Sends `PING` every interval; a peer that sent nothing for 3 intervals is dead.
async fn heartbeat(
    connection: Arc<Connection>,
    interval: Duration,
    read_task: tokio::task::AbortHandle,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.tick().await;
    let mut last_seen = connection.rx_frames.load(Ordering::Relaxed);
    let mut silent = 0;
    let mut nonce = 0u64;
    loop {
        ticker.tick().await;
        let seen = connection.rx_frames.load(Ordering::Relaxed);
        if seen == last_seen {
            silent += 1;
            if silent >= 3 {
                log::warn!(
                    "Apus action {} missed 3 heartbeats, closing",
                    connection.action
                );
                connection
                    .status
                    .store(status::NOT_RESPONDING, Ordering::Relaxed);
                read_task.abort();
                return;
            }
        } else {
            silent = 0;
            last_seen = seen;
        }
        nonce += 1;
        if connection
            .tx
            .send(message::encode_ping(kind::PING, nonce))
            .is_err()
        {
            return;
        }
    }
}

/// Once a second: ends requests past their deadline and resets idle incoming
/// streams (no timer per request or stream).
async fn sweep(connection: Arc<Connection>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        let now = Instant::now();
        for (id, request) in connection
            .request_tasks
            .drain_where(|r| r.deadline.is_some_and(|deadline| deadline <= now))
        {
            request.abort.abort();
            let error = ErrorInfo::new(
                "APUS-DEADLINE",
                "DEADLINE_EXCEEDED",
                "request did not finish before its deadline",
            );
            let _ = connection.tx.send(error.frame(request.error_kind, id));
        }
        let expired = connection
            .lock_incoming()
            .expire(Duration::from_millis(limits::STREAM_IDLE_TIMEOUT_MS));
        for id in expired {
            let _ = connection
                .tx
                .send(message::encode_reset(id, reset::TIMED_OUT));
        }
    }
}

fn protocol_error(text: &str) -> std::io::Error {
    invalid(&format!("protocol error: {text}"))
}

async fn read_loop(
    mut reader: BufReader<BoxRead>,
    connection: Arc<Connection>,
    exec: Arc<ExecContext>,
    settings: Arc<ServerSettings>,
    hub: Arc<ApusHub>,
) -> std::io::Result<()> {
    while let Some(frame) = read_frame_limited(&mut reader, settings.max_frame as usize).await? {
        connection.rx_frames.fetch_add(1, Ordering::Relaxed);
        if !kind::sent_by_action(frame.kind) {
            if kind::is_known(frame.kind) {
                return Err(protocol_error(&format!(
                    "an action may not send kind 0x{:02x}",
                    frame.kind
                )));
            }
            continue; // unknown kinds are ignored (forward compatibility)
        }
        let mut r = Reader::new(&frame.body);
        match frame.kind {
            kind::RETURN => {
                let started_at = r.i64().map_err(decode_err)?;
                let finished_at = r.i64().map_err(decode_err)?;
                if frame.body.get(16) == Some(&apus_protocol::value::tag::FUNCTION) {
                    r.u8().map_err(decode_err)?;
                    let function = convert::read_function(&mut r).map_err(decode_err)?;
                    let value = NodeValueExecutionResult {
                        value: Some(node_value_execution_result::Value::SubFlowFunctionResult(
                            function,
                        )),
                    };
                    connection.complete(
                        frame.id,
                        CallReply::Ok {
                            value,
                            started_at,
                            finished_at,
                        },
                    );
                    continue;
                }
                let mut streams = Vec::new();
                let value = convert::read_value(&mut r, &mut streams).map_err(decode_err)?;
                if streams.is_empty() {
                    let value = raw_value_result(value);
                    connection.complete(
                        frame.id,
                        CallReply::Ok {
                            value,
                            started_at,
                            finished_at,
                        },
                    );
                    continue;
                }
                let receivers = connection.expect_streams(&streams);
                let connection = connection.clone();
                tokio::spawn(async move {
                    let mut values = [value];
                    let outcome = match receivers {
                        Ok(receivers) => complete_streams(receivers, &mut values).await,
                        Err(text) => Err(text),
                    };
                    let reply = match outcome {
                        Ok(()) => CallReply::Ok {
                            value: raw_value_result(values.into_iter().next().unwrap()),
                            started_at,
                            finished_at,
                        },
                        Err(text) => CallReply::Err {
                            error: ErrorInfo::new("T-APUS-000005", "STREAM", text),
                            started_at,
                            finished_at,
                        },
                    };
                    connection.complete(frame.id, reply);
                });
            }
            kind::ERROR => {
                let started_at = r.i64().map_err(decode_err)?;
                let finished_at = r.i64().map_err(decode_err)?;
                let error = ErrorInfo::read(&mut r).map_err(decode_err)?;
                connection.complete(
                    frame.id,
                    CallReply::Err {
                        error,
                        started_at,
                        finished_at,
                    },
                );
            }
            kind::EXEC => {
                let flow_id = r.i64().map_err(decode_err)?;
                let project_id = r.i64().map_err(decode_err)?;
                let execution_id = r.str().map_err(decode_err)?.to_string();
                let deadline_ms = r.u32().map_err(decode_err)?;
                let mut streams = Vec::new();
                let payload = convert::read_value(&mut r, &mut streams).map_err(decode_err)?;
                if let Some(error) = admission_error(&connection, &hub, frame.id, Some(project_id))
                {
                    reset_streams(&connection, &streams);
                    let _ = connection.tx.send(error.frame(kind::EXEC_ERROR, frame.id));
                    continue;
                }
                let receivers = (!streams.is_empty()).then(|| connection.expect_streams(&streams));
                let deadline = (deadline_ms > 0)
                    .then(|| Instant::now() + Duration::from_millis(deadline_ms as u64));
                start_request(&connection, frame.id, kind::EXEC_ERROR, deadline, {
                    let connection = connection.clone();
                    let exec = exec.clone();
                    async move {
                        let mut values = [payload];
                        if let Some(receivers) = receivers {
                            let receivers = receivers.map_err(|text| {
                                ExecFailure::new("T-APUS-000005", "STREAM", text)
                            })?;
                            complete_streams(receivers, &mut values)
                                .await
                                .map_err(|text| {
                                    ExecFailure::new("T-APUS-000005", "STREAM", text)
                                })?;
                        }
                        let [payload] = values;
                        exec.execute(
                            &connection.action,
                            flow_id,
                            project_id,
                            &execution_id,
                            payload,
                            connection.projects.as_ref(),
                        )
                        .await
                    }
                });
            }
            kind::SUBFLOW => {
                let sub_flow_id = r.str().map_err(decode_err)?.to_string();
                let argc = r.u8().map_err(decode_err)? as usize;
                let mut streams = Vec::new();
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(convert::read_value(&mut r, &mut streams).map_err(decode_err)?);
                }
                if let Some(error) = admission_error(&connection, &hub, frame.id, None) {
                    reset_streams(&connection, &streams);
                    let _ = connection
                        .tx
                        .send(error.frame(kind::SUBFLOW_ERROR, frame.id));
                    continue;
                }
                let receivers = (!streams.is_empty()).then(|| connection.expect_streams(&streams));
                start_request(&connection, frame.id, kind::SUBFLOW_ERROR, None, {
                    let exec = exec.clone();
                    async move {
                        if let Some(receivers) = receivers {
                            let receivers = receivers.map_err(|text| {
                                ExecFailure::new("T-APUS-000005", "STREAM", text)
                            })?;
                            complete_streams(receivers, &mut args)
                                .await
                                .map_err(|text| {
                                    ExecFailure::new("T-APUS-000005", "STREAM", text)
                                })?;
                        }
                        run_sub_flow(&exec, &sub_flow_id, args).await
                    }
                });
            }
            kind::CANCEL => {
                if let Some(request) = connection.request_tasks.remove(frame.id) {
                    request.abort.abort();
                }
            }
            kind::STREAM_DATA => {
                let outcome = connection.lock_incoming().data(frame.id, r.rest());
                let reply = match outcome {
                    DataOutcome::Credit(bytes) => message::encode_credit(frame.id, bytes),
                    DataOutcome::Reset(code) => message::encode_reset(frame.id, code),
                };
                let _ = connection.tx.send(reply);
            }
            kind::STREAM_END => {
                if !connection.lock_incoming().end(frame.id) {
                    let _ = connection
                        .tx
                        .send(message::encode_reset(frame.id, reset::UNKNOWN_STREAM));
                }
            }
            kind::STREAM_RESET => {
                let code = r.u16().unwrap_or(reset::PROTOCOL_ERROR);
                if Side::Server.owns(frame.id) {
                    connection.outgoing.reset(frame.id);
                } else {
                    connection.lock_incoming().fail(frame.id, code);
                }
            }
            kind::STREAM_CREDIT => {
                let bytes = r.u32().map_err(decode_err)?;
                if !Side::Server.owns(frame.id) {
                    return Err(protocol_error(
                        "credit for a stream the server did not open",
                    ));
                }
                connection.outgoing.credit(frame.id, bytes);
            }
            kind::PING => {
                let mut pong = FrameBuilder::with_capacity(kind::PONG, frame.id, frame.body.len());
                pong.body().buf.extend_from_slice(&frame.body);
                let _ = connection.tx.send(pong.finish());
            }
            kind::PONG => {}
            kind::GOAWAY => {
                log::info!("Apus action {} is draining", connection.action);
                connection.draining.store(true, Ordering::Relaxed);
            }
            kind::STATUS => {
                let value = r.u8().map_err(decode_err)?;
                if !matches!(value, status::NOT_READY | status::RUNNING | status::STOPPED) {
                    return Err(protocol_error("invalid STATUS"));
                }
                connection.status.store(value, Ordering::Relaxed);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks an action request before it starts: duplicate id, routing, ticket,
/// admission control. `None` = accepted (and counted as in flight).
fn admission_error(
    connection: &Arc<Connection>,
    hub: &ApusHub,
    id: u32,
    project_id: Option<i64>,
) -> Option<ErrorInfo> {
    if connection.request_tasks.contains(id) {
        return Some(ErrorInfo::new(
            "APUS-DUPLICATE-ID",
            "INVALID_ARGUMENT",
            "request id already in flight",
        ));
    }
    if let Some(project_id) = project_id {
        if !hub.serves_project(project_id) {
            return Some(ErrorInfo::new(
                "APUS-WRONG-INSTANCE",
                "UNAVAILABLE",
                format!("project {project_id} is served by another instance"),
            ));
        }
        if !connection.serves(project_id) {
            return Some(ErrorInfo::new(
                "APUS-FORBIDDEN",
                "PERMISSION_DENIED",
                format!("the ticket does not cover project {project_id}"),
            ));
        }
    }
    if connection
        .requests_in_flight
        .fetch_add(1, Ordering::Relaxed)
        >= connection.max_in_flight
    {
        connection
            .requests_in_flight
            .fetch_sub(1, Ordering::Relaxed);
        if !connection.overloaded.swap(true, Ordering::Relaxed) {
            let _ = connection
                .tx
                .send(message::encode_status(status::NOT_READY));
        }
        return Some(ErrorInfo::new(
            "APUS-OVERLOADED",
            "UNAVAILABLE",
            "too many requests in flight",
        ));
    }
    None
}

fn reset_streams(connection: &Connection, streams: &[(u32, i64)]) {
    for (id, _) in streams {
        let _ = connection
            .tx
            .send(message::encode_reset(*id, reset::CANCELLED));
    }
}

/// Runs one admitted action request (`EXEC` / `SUBFLOW`) with cancellation and
/// a deadline, and answers with the matching result frame.
fn start_request<F>(
    connection: &Arc<Connection>,
    id: u32,
    error_kind: u8,
    deadline: Option<Instant>,
    work: F,
) where
    F: std::future::Future<Output = Result<Value, ExecFailure>> + Send + 'static,
{
    let result_kind = error_kind - 1; // EXEC_RESULT / SUBFLOW_RESULT

    struct InFlight(Arc<Connection>, u32);
    impl Drop for InFlight {
        fn drop(&mut self) {
            let connection = &self.0;
            let left = connection
                .requests_in_flight
                .fetch_sub(1, Ordering::Relaxed)
                - 1;
            connection.request_tasks.remove(self.1);
            // Hysteresis: running again once more than half the slots are free.
            if left.saturating_mul(2) < connection.max_in_flight
                && connection.overloaded.swap(false, Ordering::Relaxed)
            {
                let _ = connection.tx.send(message::encode_status(status::RUNNING));
            }
        }
    }

    let guard = InFlight(connection.clone(), id);
    let task_connection = connection.clone();
    // Hold the entry's shard while spawning so the task cannot remove its entry
    // before it has been inserted.
    let mut shard = connection.request_tasks.shard(id);
    let task = tokio::spawn(async move {
        let _guard = guard;
        let connection = task_connection;
        let mut streams = StreamOut::new(connection.stream_threshold, &connection.next_stream_id);
        let frame = match work.await {
            Ok(value) => {
                let mut f = FrameBuilder::with_capacity(result_kind, id, 64);
                convert::write_value(f.body(), &value, &mut Some(&mut streams));
                f.finish()
            }
            Err(failure) => failure.into_error().frame(error_kind, id),
        };
        let _ = connection.tx.send(frame);
        connection.send_streams(streams.pending);
    });
    shard.insert(
        id,
        RunningRequest {
            abort: task.abort_handle(),
            deadline,
            error_kind,
        },
    );
}

pub(super) async fn run_sub_flow(
    exec: &ExecContext,
    sub_flow_id: &str,
    args: Vec<Value>,
) -> Result<Value, ExecFailure> {
    match exec
        .engine
        .execute_sub_flow(sub_flow_id, args, Some(&exec.remote), exec.with_trace)
        .await
    {
        Some(report) => match report.signal {
            Signal::Success(value) | Signal::Return(value) => Ok(value),
            Signal::Stop => Ok(Value {
                kind: Some(tucana::shared::value::Kind::NullValue(0)),
            }),
            Signal::Failure(err) => Err(ExecFailure::new(&err.code, &err.category, err.message)),
        },
        None => Err(ExecFailure::new(
            "T-TAURUS-000002",
            "SubFlowExecutionNotFound",
            "No pending sub flow execution found for this execution identifier",
        )),
    }
}

/// Loads a routing table file: `{"version": n, "instances": [{"id", "endpoint", "projects": [..]}]}`.
pub fn load_routes(path: &str) -> Result<Routes, String> {
    #[derive(serde::Deserialize)]
    struct File {
        version: u64,
        instances: Vec<Instance>,
    }
    #[derive(serde::Deserialize)]
    struct Instance {
        id: String,
        endpoint: String,
        #[serde(default)]
        projects: Vec<i64>,
    }
    let data = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let file: File =
        serde_json::from_str(&data).map_err(|e| format!("cannot parse {path}: {e}"))?;
    Ok(Routes {
        version: file.version,
        routes: file
            .instances
            .into_iter()
            .map(|i| Route {
                instance_id: i.id,
                endpoint: i.endpoint,
                projects: i.projects,
            })
            .collect(),
    })
}

/// Loads the routing table and reloads it when the file changes (checked every second).
pub fn watch_routes(hub: Arc<ApusHub>, path: String, instance_id: String) {
    tokio::spawn(async move {
        let mut last_modified = None;
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
            if modified.is_none() || modified == last_modified {
                continue;
            }
            last_modified = modified;
            match load_routes(&path) {
                Ok(routes) => hub.set_routes(routes, &instance_id),
                Err(err) => log::warn!("Apus: {}", err),
            }
        }
    });
}

fn decode_err(err: apus_protocol::DecodeError) -> std::io::Error {
    protocol_error(&err.to_string())
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}
