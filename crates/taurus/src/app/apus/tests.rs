//! Integration tests of the Apus v1 server: a raw-frame test action talks to a
//! real server (real engine, flows in memory) over loopback TCP or TLS.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use apus_protocol::frame::{Frame, FrameBuilder, kind};
use apus_protocol::message::{Hello, Route, Routes, Welcome, encode_status, reset, status};
use apus_protocol::value::tag;
use apus_protocol::{ApusValue, ErrorInfo, Reader, Writer, feature, read_frame, reject, tls};
use taurus_core::runtime::engine::ExecutionEngine;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tucana::shared::number_value::Number;
use tucana::shared::value::Kind;
use tucana::shared::{
    InputType, ListValue, LiteralValue, NodeFunction, NodeParameter, NodeValue, NumberValue,
    ReferenceValue, SubFlow, ValidationFlow, Value, node_value, reference_value, sub_flow,
};

use super::exec::{ExecContext, ExecutionMode, FlowSource};
use super::hub::{ApusHub, ServerSettings, serve_on};
use super::remote::RoutingRemote;

const ACTION: &str = "test-action";
const FUNCTIONS: [&str; 2] = ["test::echo", "test::map"];

fn int(i: i64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(NumberValue {
            number: Some(Number::Integer(i)),
        })),
    }
}

fn string(s: &str) -> Value {
    Value {
        kind: Some(Kind::StringValue(s.into())),
    }
}

fn literal(db: i64, name: &str, value: Value) -> NodeParameter {
    NodeParameter {
        database_id: db,
        runtime_parameter_id: name.into(),
        value: Some(NodeValue {
            value: Some(node_value::Value::LiteralValue(LiteralValue {
                value: Some(value),
                references: vec![],
            })),
        }),
        cast: None,
    }
}

fn node(id: i64, function: &str, parameters: Vec<NodeParameter>, remote: bool) -> NodeFunction {
    NodeFunction {
        database_id: Some(id),
        runtime_function_id: function.into(),
        parameters,
        next_node_id: None,
        definition_source: remote.then(|| ACTION.to_string()),
    }
}

fn flow(flow_id: i64, project_id: i64, nodes: Vec<NodeFunction>) -> (i64, Arc<ValidationFlow>) {
    let starting_node_id = nodes[0].database_id.unwrap();
    (
        flow_id,
        Arc::new(ValidationFlow {
            flow_id,
            project_id,
            r#type: "REST".into(),
            starting_node_id,
            node_functions: nodes,
            definition_source: Some(format!("action.{ACTION}")),
            ..Default::default()
        }),
    )
}

/// 1, 2: echo a literal (projects 1 and 2). 3: echo a 100 KiB string. 4: map over [1, 2] with a sub-flow (+100).
/// 5: the action returns a function, which Taurus's `std::list::map` runs over [1, 2, 3].
fn flows() -> HashMap<i64, Arc<ValidationFlow>> {
    let sub_flow = NodeParameter {
        database_id: 42,
        runtime_parameter_id: "consumer".into(),
        value: Some(NodeValue {
            value: Some(node_value::Value::SubFlow(SubFlow {
                execution_reference: Some(sub_flow::ExecutionReference::StartingNodeId(43)),
                ..Default::default()
            })),
        }),
        cast: None,
    };
    let add = node(
        43,
        "std::number::add",
        vec![
            NodeParameter {
                database_id: 431,
                runtime_parameter_id: "first".into(),
                value: Some(NodeValue {
                    value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                        target: Some(reference_value::Target::InputType(InputType {
                            node_id: 41,
                            parameter_index: 1,
                            input_index: 0,
                        })),
                        paths: vec![],
                    })),
                }),
                cast: None,
            },
            literal(432, "second", int(100)),
        ],
        false,
    );
    HashMap::from([
        flow(
            1,
            1,
            vec![node(
                11,
                "test::echo",
                vec![literal(111, "value", string("hello"))],
                true,
            )],
        ),
        flow(
            2,
            2,
            vec![node(
                21,
                "test::echo",
                vec![literal(211, "value", string("other project"))],
                true,
            )],
        ),
        flow(
            3,
            1,
            vec![node(
                31,
                "test::echo",
                vec![literal(311, "value", string(&"x".repeat(100 * 1024)))],
                true,
            )],
        ),
        flow(
            5,
            1,
            vec![
                NodeFunction {
                    next_node_id: Some(52),
                    ..node(
                        51,
                        "test::echo",
                        vec![literal(511, "value", string("square"))],
                        true,
                    )
                },
                node(
                    52,
                    "std::list::map",
                    vec![
                        literal(
                            521,
                            "list",
                            Value {
                                kind: Some(Kind::ListValue(ListValue {
                                    values: vec![int(1), int(2), int(3)],
                                })),
                            },
                        ),
                        NodeParameter {
                            database_id: 522,
                            runtime_parameter_id: "transform".into(),
                            value: Some(NodeValue {
                                value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                                    target: Some(reference_value::Target::NodeId(51)),
                                    paths: vec![],
                                })),
                            }),
                            cast: None,
                        },
                    ],
                    false,
                ),
            ],
        ),
        flow(
            4,
            1,
            vec![
                node(
                    41,
                    "test::map",
                    vec![
                        literal(
                            411,
                            "items",
                            Value {
                                kind: Some(Kind::ListValue(ListValue {
                                    values: vec![int(1), int(2)],
                                })),
                            },
                        ),
                        sub_flow,
                    ],
                    true,
                ),
                add,
            ],
        ),
    ])
}

struct Server {
    port: u16,
    hub: Arc<ApusHub>,
}

struct Options {
    max_in_flight: u32,
    heartbeat: Duration,
    tls: Option<tls::TlsAcceptor>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_in_flight: 64,
            heartbeat: Duration::from_secs(30),
            tls: None,
        }
    }
}

async fn start(options: Options) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let hub = Arc::new(ApusHub::new("taurus-a", &format!("127.0.0.1:{port}")));
    let exec = Arc::new(ExecContext {
        client: None,
        flows: FlowSource::Memory(flows()),
        mode: ExecutionMode::Local,
        engine: Arc::new(ExecutionEngine::new()),
        remote: RoutingRemote::new(Some(hub.clone()), None, Duration::from_secs(5), false),
        timeout: Duration::from_secs(5),
        flow_type: "static".into(),
        with_trace: false,
        runtime_usage: std::sync::Mutex::new(None),
    });
    let settings = ServerSettings {
        host: "127.0.0.1".into(),
        port,
        instance_id: "taurus-a".into(),
        max_in_flight: options.max_in_flight,
        heartbeat: options.heartbeat,
        max_frame: 1 << 20,
        stream_window: 64 * 1024,
        stream_threshold: 1024,
        tls: options.tls,
        max_connections: 100,
        max_connections_per_action: 4,
        handshake_timeout: Duration::from_millis(300),
    };
    tokio::spawn(serve_on(listener, settings, hub.clone(), exec));
    Server { port, hub }
}

/// A raw-frame test action.
struct Client {
    reader: BufReader<Box<dyn AsyncRead + Send + Unpin>>,
    writer: Box<dyn AsyncWrite + Send + Unpin>,
}

fn hello() -> Vec<u8> {
    Hello {
        version: 1,
        action: ACTION.into(),
        scaling: 0,
        max_in_flight: 0,
        max_frame: 1 << 20,
        stream_window: 64 * 1024,
        features: feature::ALL,
        functions: FUNCTIONS.map(String::from).to_vec(),
    }
    .encode()
}

impl Client {
    async fn raw(port: u16) -> Self {
        let tcp = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        tcp.set_nodelay(true).unwrap();
        let (r, w) = tcp.into_split();
        Self {
            reader: BufReader::new(Box::new(r)),
            writer: Box::new(w),
        }
    }

    async fn tls(port: u16, connector: tls::TlsConnector) -> Self {
        let tcp = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let stream = connector
            .connect(tls::server_name("localhost:0").unwrap(), tcp)
            .await
            .unwrap();
        let (r, w) = tokio::io::split(stream);
        Self {
            reader: BufReader::new(Box::new(r)),
            writer: Box::new(w),
        }
    }

    /// Connects, authenticates and reports `running`; returns the WELCOME.
    async fn connect(port: u16) -> (Self, Welcome) {
        let mut client = Self::raw(port).await;
        let welcome = client.handshake().await;
        (client, welcome)
    }

    async fn handshake(&mut self) -> Welcome {
        self.send(hello()).await;
        let welcome = self.recv().await.expect("WELCOME");
        assert_eq!(welcome.kind, kind::WELCOME);
        let routes = self.recv().await.expect("ROUTES");
        assert_eq!(routes.kind, kind::ROUTES);
        self.send(encode_status(status::RUNNING)).await;
        Welcome::decode(&welcome.body).unwrap()
    }

    async fn send(&mut self, frame: Vec<u8>) {
        self.writer.write_all(&frame).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    /// Next frame other than PING/STREAM_CREDIT, or None on close/timeout.
    async fn recv(&mut self) -> Option<Frame> {
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(5), read_frame(&mut self.reader))
                .await
                .ok()?
                .ok()??;
            if !matches!(frame.kind, kind::PING | kind::STREAM_CREDIT) {
                return Some(frame);
            }
        }
    }

    async fn exec(&mut self, id: u32, flow_id: i64, project_id: i64, deadline_ms: u32) {
        let mut f = FrameBuilder::new(kind::EXEC, id);
        let w = f.body();
        w.i64(flow_id);
        w.i64(project_id);
        w.str(&format!("00000000-0000-4000-8000-{id:012}"));
        w.u32(deadline_ms);
        w.null();
        self.send(f.finish()).await;
    }

    async fn reply(&mut self, call_id: u32, value: &ApusValue) {
        let mut f = FrameBuilder::new(kind::RETURN, call_id);
        f.body().i64(1);
        f.body().i64(2);
        f.body().value(value);
        self.send(f.finish()).await;
    }

    /// Reads a CALL: (id, method, args).
    async fn call(&mut self) -> (u32, u16, Vec<ApusValue>) {
        let frame = self.recv().await.expect("CALL");
        assert_eq!(
            frame.kind,
            kind::CALL,
            "expected CALL, got 0x{:02x}",
            frame.kind
        );
        let mut r = Reader::new(&frame.body);
        let method = r.u16().unwrap();
        let _execution = r.str().unwrap();
        let _project = r.i64().unwrap();
        let _deadline = r.u32().unwrap();
        let argc = r.u8().unwrap();
        (
            frame.id,
            method,
            (0..argc).map(|_| r.value().unwrap()).collect(),
        )
    }

    async fn closed(&mut self) -> bool {
        loop {
            match tokio::time::timeout(Duration::from_secs(5), read_frame(&mut self.reader)).await {
                Ok(Ok(None)) | Ok(Err(_)) => return true,
                Ok(Ok(Some(_))) => continue,
                Err(_) => return false,
            }
        }
    }
}

fn error_of(frame: &Frame) -> ErrorInfo {
    ErrorInfo::read(&mut Reader::new(&frame.body)).unwrap()
}

fn value_of(frame: &Frame) -> ApusValue {
    Reader::new(&frame.body).value().unwrap()
}

#[tokio::test]
async fn rejects_bad_handshakes() {
    let server = start(Options::default()).await;
    for (frame, code) in [(
        apus_protocol::message::encode_ping(kind::PING, 1),
        reject::PROTOCOL_ERROR,
    )] {
        let mut client = Client::raw(server.port).await;
        client.send(frame).await;
        let reject = client.recv().await.expect("REJECT");
        assert_eq!(reject.kind, kind::REJECT);
        assert_eq!(
            apus_protocol::message::decode_reject(&reject.body)
                .unwrap()
                .0,
            code
        );
        assert!(client.closed().await);
    }
}

#[tokio::test]
async fn closes_connections_without_hello() {
    let server = start(Options::default()).await;
    let mut client = Client::raw(server.port).await;
    let started = std::time::Instant::now();
    assert!(client.closed().await);
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn executes_a_flow_that_calls_the_action() {
    let server = start(Options::default()).await;
    let (mut client, welcome) = Client::connect(server.port).await;
    assert_eq!(welcome.instance_id, "taurus-a");
    client.exec(1, 1, 1, 5_000).await;
    let (call, method, args) = client.call().await;
    assert_eq!((method, args), (0, vec![ApusValue::string("hello")]));
    client.reply(call, &ApusValue::string("echoed")).await;
    let result = client.recv().await.unwrap();
    assert_eq!(result.kind, kind::EXEC_RESULT);
    assert_eq!(value_of(&result), ApusValue::string("echoed"));
}

#[tokio::test]
async fn a_returned_function_is_run_by_taurus() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 5, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client
        .reply(call, &ApusValue::function("std::number::square", ""))
        .await;
    let result = client.recv().await.unwrap();
    assert_eq!(result.kind, kind::EXEC_RESULT, "{:?}", error_of(&result));
    assert_eq!(
        value_of(&result),
        ApusValue::List(vec![
            ApusValue::Int(1),
            ApusValue::Int(4),
            ApusValue::Int(9)
        ])
    );
}

#[tokio::test]
async fn calls_only_reach_running_actions() {
    let server = start(Options::default()).await;
    let mut client = Client::raw(server.port).await;
    client.send(hello()).await;
    client.recv().await.unwrap(); // WELCOME
    client.recv().await.unwrap(); // ROUTES, no STATUS running
    client.exec(1, 1, 1, 5_000).await;
    let result = client.recv().await.unwrap();
    assert_eq!(result.kind, kind::EXEC_ERROR);
    assert_eq!(error_of(&result).code, "T-APUS-000003");
}

#[tokio::test]
async fn routes_projects_to_their_instance() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    server.hub.set_routes(
        Routes {
            version: 2,
            routes: vec![
                Route {
                    instance_id: "taurus-a".into(),
                    endpoint: format!("127.0.0.1:{}", server.port),
                    projects: vec![1],
                },
                Route {
                    instance_id: "taurus-b".into(),
                    endpoint: "127.0.0.1:1".into(),
                    projects: vec![2],
                },
            ],
        },
        "taurus-a",
    );
    let routes = client.recv().await.unwrap();
    assert_eq!(routes.kind, kind::ROUTES);
    let routes = Routes::decode(&routes.body).unwrap();
    assert_eq!(routes.instance_for(2).unwrap().instance_id, "taurus-b");
    client.exec(1, 2, 2, 5_000).await;
    let result = client.recv().await.unwrap();
    assert_eq!(error_of(&result).code, "APUS-WRONG-INSTANCE");
    // A flow claimed for the wrong project is refused too.
    client.exec(2, 2, 1, 5_000).await;
    assert_eq!(
        error_of(&client.recv().await.unwrap()).code,
        "T-APUS-FLOW-000004"
    );
}

#[tokio::test]
async fn the_default_route_serves_unlisted_projects() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    // taurus-a has an empty list (default route); taurus-b takes project 2.
    server.hub.set_routes(
        Routes {
            version: 2,
            routes: vec![
                Route {
                    instance_id: "taurus-a".into(),
                    endpoint: format!("127.0.0.1:{}", server.port),
                    projects: vec![],
                },
                Route {
                    instance_id: "taurus-b".into(),
                    endpoint: "127.0.0.1:1".into(),
                    projects: vec![2],
                },
            ],
        },
        "taurus-a",
    );
    assert_eq!(
        client.recv().await.unwrap().kind,
        kind::ROUTES,
        "no GOAWAY: the default route still serves projects"
    );
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client.reply(call, &ApusValue::Null).await;
    assert_eq!(client.recv().await.unwrap().kind, kind::EXEC_RESULT);
    client.exec(2, 2, 2, 5_000).await;
    assert_eq!(
        error_of(&client.recv().await.unwrap()).code,
        "APUS-WRONG-INSTANCE"
    );
    // Dropped from the table: the instance asks its actions to leave.
    server.hub.set_routes(
        Routes {
            version: 3,
            routes: vec![Route {
                instance_id: "taurus-b".into(),
                endpoint: "127.0.0.1:1".into(),
                projects: vec![],
            }],
        },
        "taurus-a",
    );
    assert_eq!(client.recv().await.unwrap().kind, kind::ROUTES);
    assert_eq!(client.recv().await.unwrap().kind, kind::GOAWAY);
}

#[tokio::test]
async fn rejects_duplicate_ids_and_overload() {
    let server = start(Options {
        max_in_flight: 1,
        ..Default::default()
    })
    .await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(7, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client.exec(7, 1, 1, 5_000).await;
    assert_eq!(
        error_of(&client.recv().await.unwrap()).code,
        "APUS-DUPLICATE-ID"
    );
    client.exec(8, 1, 1, 5_000).await;
    let status_frame = client.recv().await.unwrap();
    assert_eq!(
        (status_frame.kind, status_frame.body[0]),
        (kind::STATUS, status::NOT_READY)
    );
    assert_eq!(
        error_of(&client.recv().await.unwrap()).code,
        "APUS-OVERLOADED"
    );
    client.reply(call, &ApusValue::Null).await;
    let mut kinds = vec![client.recv().await.unwrap(), client.recv().await.unwrap()];
    kinds.sort_by_key(|f| f.kind);
    assert_eq!(kinds[0].kind, kind::STATUS);
    assert_eq!(kinds[0].body[0], status::RUNNING);
    assert_eq!(kinds[1].kind, kind::EXEC_RESULT);
}

#[tokio::test]
async fn protocol_violations_close_only_that_connection() {
    let server = start(Options::default()).await;
    // An action may not send CALL.
    let (mut client, _) = Client::connect(server.port).await;
    client.send(FrameBuilder::new(kind::CALL, 1).finish()).await;
    assert!(client.closed().await);
    // A payload nested 10 000 levels deep.
    let (mut client, _) = Client::connect(server.port).await;
    let mut f = FrameBuilder::new(kind::EXEC, 1);
    let w = f.body();
    w.i64(1);
    w.i64(1);
    w.str("e");
    w.u32(0);
    for _ in 0..10_000 {
        w.u8(tag::LIST);
        w.u32(1);
    }
    w.null();
    client.send(f.finish()).await;
    assert!(client.closed().await);
    // The server still works for others.
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client.reply(call, &ApusValue::Null).await;
    assert_eq!(client.recv().await.unwrap().kind, kind::EXEC_RESULT);
}

#[tokio::test]
async fn streams_from_the_action_are_materialised() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    let mut f = FrameBuilder::new(kind::RETURN, call);
    f.body().i64(1);
    f.body().i64(2);
    f.body().stream(1, apus_protocol::value::TEXT_STREAM, 6);
    client.send(f.finish()).await;
    for chunk in ["str", "eam"] {
        let mut data = FrameBuilder::new(kind::STREAM_DATA, 1);
        data.body().buf.extend_from_slice(chunk.as_bytes());
        client.send(data.finish()).await;
    }
    client
        .send(apus_protocol::frame::empty_frame(kind::STREAM_END, 1))
        .await;
    let result = client.recv().await.unwrap();
    assert_eq!(value_of(&result), ApusValue::string("stream"));
}

#[tokio::test]
async fn refuses_streams_with_the_wrong_parity() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    let mut f = FrameBuilder::new(kind::RETURN, call);
    f.body().i64(1);
    f.body().i64(2);
    f.body().stream(2, apus_protocol::value::TEXT_STREAM, 6); // even = server-owned id
    client.send(f.finish()).await;
    let reset_frame = client.recv().await.unwrap();
    assert_eq!(
        (
            reset_frame.kind,
            u16::from_le_bytes([reset_frame.body[0], reset_frame.body[1]])
        ),
        (kind::STREAM_RESET, reset::PROTOCOL_ERROR)
    );
    let result = client.recv().await.unwrap();
    assert_eq!(result.kind, kind::EXEC_ERROR);
}

#[tokio::test]
async fn large_params_are_streamed_to_the_action() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 3, 1, 5_000).await;
    let (call, _, args) = client.call().await;
    let ApusValue::Stream { id, length, .. } = args[0] else {
        panic!("expected a stream, got {:?}", args[0])
    };
    assert_eq!((id % 2, length), (0, 100 * 1024));
    let mut received = Vec::new();
    loop {
        let frame = client.recv().await.unwrap();
        match frame.kind {
            kind::STREAM_DATA => {
                received.extend_from_slice(&frame.body);
                client
                    .send(apus_protocol::message::encode_credit(
                        id,
                        frame.body.len() as u32,
                    ))
                    .await;
            }
            kind::STREAM_END => break,
            other => panic!("unexpected frame 0x{other:02x}"),
        }
    }
    assert_eq!(received, vec![b'x'; 100 * 1024]);
    client
        .reply(call, &ApusValue::Int(received.len() as i64))
        .await;
    assert_eq!(
        value_of(&client.recv().await.unwrap()),
        ApusValue::Int(100 * 1024)
    );
}

#[tokio::test]
async fn sub_flows_run_on_the_same_connection() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 4, 1, 5_000).await;
    let (call, method, args) = client.call().await;
    assert_eq!(method, 1);
    let ApusValue::SubFlow { id: handle, .. } = &args[1] else {
        panic!("expected a sub-flow handle")
    };
    let mut results = Vec::new();
    for (request, item) in [(10u32, 1), (11, 2)] {
        let mut f = FrameBuilder::new(kind::SUBFLOW, request);
        f.body().str(handle);
        f.body().u8(1);
        f.body().int(item);
        client.send(f.finish()).await;
        let result = client.recv().await.unwrap();
        assert_eq!((result.kind, result.id), (kind::SUBFLOW_RESULT, request));
        results.push(value_of(&result));
    }
    assert_eq!(results, vec![ApusValue::Int(101), ApusValue::Int(102)]);
    client.reply(call, &ApusValue::List(results)).await;
    assert_eq!(client.recv().await.unwrap().kind, kind::EXEC_RESULT);
}

#[tokio::test]
async fn deadlines_end_requests() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 1, 1, 100).await;
    client.call().await; // never answered
    let started = std::time::Instant::now();
    let result = client.recv().await.unwrap();
    assert_eq!(error_of(&result).code, "APUS-DEADLINE");
    assert!(started.elapsed() < Duration::from_millis(1500));
}

#[tokio::test]
async fn cancel_stops_the_execution_and_its_call() {
    let server = start(Options::default()).await;
    let (mut client, _) = Client::connect(server.port).await;
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client
        .send(apus_protocol::message::encode_cancel(
            1,
            apus_protocol::cancel::CALLER,
        ))
        .await;
    // Taurus cancels the outstanding CALL in turn and never answers the EXEC.
    let cancel = client.recv().await.unwrap();
    assert_eq!((cancel.kind, cancel.id), (kind::CANCEL, call));
    client.reply(call, &ApusValue::Null).await; // late answer: ignored
    assert!(
        tokio::time::timeout(Duration::from_millis(300), client.recv())
            .await
            .unwrap_or(None)
            .is_none()
    );
}

#[tokio::test]
async fn silent_actions_are_disconnected() {
    let server = start(Options {
        heartbeat: Duration::from_millis(100),
        ..Default::default()
    })
    .await;
    let (mut client, welcome) = Client::connect(server.port).await;
    assert_eq!(welcome.heartbeat_ms, 100);
    let started = std::time::Instant::now();
    assert!(client.closed().await);
    assert!(started.elapsed() < Duration::from_secs(2));
}

/// Aquila's role in the tests: a CA that issues server and action certificates.
struct TestCa {
    cert: rcgen::Certificate,
    key: rcgen::KeyPair,
}

impl TestCa {
    fn new() -> Self {
        let key = rcgen::KeyPair::generate().unwrap();
        let mut params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let cert = params.self_signed(&key).unwrap();
        Self { cert, key }
    }

    fn acceptor(&self) -> tls::TlsAcceptor {
        let key = rcgen::KeyPair::generate().unwrap();
        let cert = rcgen::CertificateParams::new(vec!["localhost".to_string()])
            .unwrap()
            .signed_by(&key, &self.cert, &self.key)
            .unwrap();
        let ca = self.cert.pem();
        let config = tls::server_config(
            cert.pem().as_bytes(),
            key.serialize_pem().as_bytes(),
            Some(ca.as_bytes()),
        )
        .unwrap();
        tls::TlsAcceptor::from(config)
    }

    /// An action certificate; `identity` = the URI SAN, `valid_for` = time until it expires.
    fn connector(&self, identity: Option<&str>, valid_for: Option<Duration>) -> tls::TlsConnector {
        let key = rcgen::KeyPair::generate().unwrap();
        let mut params = rcgen::CertificateParams::new(vec!["action.local".to_string()]).unwrap();
        if let Some(uri) = identity {
            params.subject_alt_names.push(rcgen::SanType::URI(
                rcgen::Ia5String::try_from(uri).unwrap(),
            ));
        }
        if let Some(valid_for) = valid_for {
            params.not_after = time::OffsetDateTime::now_utc() + valid_for;
        }
        let cert = params.signed_by(&key, &self.cert, &self.key).unwrap();
        let ca = self.cert.pem();
        let config = tls::client_config(
            ca.as_bytes(),
            Some((cert.pem().as_bytes(), key.serialize_pem().as_bytes())),
        )
        .unwrap();
        tls::TlsConnector::from(config)
    }
}

#[tokio::test]
async fn authenticates_actions_by_client_certificate() {
    let ca = TestCa::new();
    let server = start(Options {
        tls: Some(ca.acceptor()),
        ..Default::default()
    })
    .await;

    // The certificate names this action: calls work.
    let identity = tls::identity_uri(ACTION, None, 1);
    let mut client = Client::tls(server.port, ca.connector(Some(&identity), None)).await;
    client.handshake().await;
    client.exec(1, 1, 1, 5_000).await;
    let (call, _, _) = client.call().await;
    client.reply(call, &ApusValue::Bool(true)).await;
    assert_eq!(
        value_of(&client.recv().await.unwrap()),
        ApusValue::Bool(true)
    );

    // A certificate for another action, or one without an identity: REJECT.
    let other = tls::identity_uri("other-action", None, 1);
    for connector in [ca.connector(Some(&other), None), ca.connector(None, None)] {
        let mut client = Client::tls(server.port, connector).await;
        client.send(hello()).await;
        let frame = client.recv().await.expect("REJECT");
        assert_eq!(frame.kind, kind::REJECT);
        assert_eq!(
            apus_protocol::message::decode_reject(&frame.body)
                .unwrap()
                .0,
            reject::UNAUTHENTICATED
        );
    }

    // No client certificate, or plain TCP: the TLS handshake fails.
    let without =
        tls::TlsConnector::from(tls::client_config(ca.cert.pem().as_bytes(), None).unwrap());
    let mut client = Client::tls(server.port, without).await;
    client.send(hello()).await;
    assert!(client.closed().await);
    let mut plain = Client::raw(server.port).await;
    plain.send(hello()).await;
    assert!(plain.closed().await);
}

#[tokio::test]
async fn connections_end_when_the_certificate_expires() {
    let ca = TestCa::new();
    let server = start(Options {
        tls: Some(ca.acceptor()),
        ..Default::default()
    })
    .await;
    let identity = tls::identity_uri(ACTION, Some(&[1]), 1);
    let mut client = Client::tls(
        server.port,
        ca.connector(Some(&identity), Some(Duration::from_secs(2))),
    )
    .await;
    client.handshake().await;
    let frame = client.recv().await.expect("GOAWAY");
    assert_eq!(frame.kind, kind::GOAWAY);
    assert!(
        server.hub.pick(ACTION, "test::echo", 1).is_none(),
        "an expired connection gets no calls"
    );
}

#[test]
fn loads_routing_files() {
    let path = std::env::temp_dir().join(format!("apus-routes-{}.json", std::process::id()));
    std::fs::write(&path, r#"{"version": 3, "instances": [{"id": "a", "endpoint": "h:1", "projects": [1, 2]}, {"id": "b", "endpoint": "h:2"}]}"#).unwrap();
    let routes = super::hub::load_routes(path.to_str().unwrap()).unwrap();
    std::fs::remove_file(path).ok();
    assert_eq!(routes.version, 3);
    assert_eq!(routes.instance_for(2).unwrap().instance_id, "a");
    assert_eq!(
        routes.instance_for(9).unwrap().instance_id,
        "b",
        "an instance without projects is the default route"
    );
    let _ = Writer::default();
}
