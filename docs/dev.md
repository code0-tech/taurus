---
title: Taurus Development Guide
---

This guide is for contributors working on Taurus itself.
It documents how Taurus is structured, how execution flows through the runtime, and how to run and test changes locally.

## What Taurus Does

Taurus is the execution runtime in the CodeZero execution block.

- Consumes flow execution requests from NATS (`execution.*`)
- Executes flow graphs via `taurus-core::runtime::engine::ExecutionEngine`
- Emits lifecycle events to NATS (`runtime.emitter.<execution_id>`)
- Delegates remote nodes to external services over NATS (`action.<service>.<execution_id>`)
- Reports runtime status and usage to Aquila in dynamic mode

## Workspace Layout

| Path | Purpose |
| --- | --- |
| `crates/taurus` | Main runtime binary (startup, config, NATS worker, dynamic integrations) |
| `crates/taurus-core` | Execution engine, compiler, runtime functions, errors, tracing |
| `crates/taurus-provider` | Transport adapters (NATS emitter + NATS remote runtime) |
| `crates/taurus-manual` | Manual CLI executor for running a single validation flow file |
| `crates/taurus-tests` | Local execution-suite runner for JSON flow fixtures under `flows/` |
| `flows/` | Example/validation flow cases used by `taurus-tests` |

## Runtime Flow

```mermaid
graph TD
  NATS[NATS]
  Taurus[Taurus Runtime\ncrates/taurus]
  Core[ExecutionEngine\ncrates/taurus-core]
  Emitter[Runtime Emitter\ncrates/taurus-provider]
  Remote[Remote Runtime Adapter\ncrates/taurus-provider]
  Service[Remote Service / Action Runtime]
  Aquila[Aquila gRPC APIs\n(dynamic mode only)]

  NATS -->|execution.*| Taurus
  Taurus --> Core
  Core --> Emitter
  Emitter -->|runtime.emitter.<execution_id>| NATS
  Core --> Remote
  Remote -->|action.<service>.<execution_id>| NATS
  NATS --> Service
  Taurus -->|runtime status + usage| Aquila
```

### Execution details

1. Taurus subscribes to queue subject `execution.*` with queue group `taurus`.
2. Incoming payload is decoded as `tucana::shared::ExecutionFlow`.
3. `ExecutionEngine::execute_flow_with_execution_id(...)` compiles and executes nodes.
4. Local nodes run handlers from the built-in function registry.
5. Non-local `definition_source` values are executed remotely via `RemoteRuntime`.
6. Lifecycle events are emitted as `starting`, `ongoing`, `finished`, or `failed`.

## Runtime Modes

Taurus mode is controlled by `MODE`.

### `dynamic`

`dynamic` enables control-plane integrations:

- Sends definitions to Aquila (retry loop until success)
- Starts runtime status reporting (including heartbeat)
- Sends runtime usage updates after each flow run

### `static`

`static` disables those control-plane interactions.

- Taurus still executes flows from NATS
- No definition push
- No runtime status updates
- No runtime usage updates

## Environment Variables

Defaults are defined in `crates/taurus/src/config/mod.rs`.

| Name | Description | Default |
| --- | --- | --- |
| `ENVIRONMENT` | Running env | `development` |
| `MODE` | Runtime mode (`dynamic` or `static`) | `dynamic` |
| `NATS_URL` | NATS connection URL | `nats://localhost:4222` |
| `AQUILA_URL` | Aquila gRPC endpoint (used in dynamic mode) | `http://localhost:50051` |
| `AQUILA_TOKEN` | Auth token for Aquila runtime APIs | `token` |
| `WITH_HEALTH_SERVICE` | Enables gRPC health server | `false` |
| `GRPC_HOST` | Health server host | `127.0.0.1` |
| `GRPC_PORT` | Health server port | `50051` |
| `DEFINITIONS` | Path sent to `FlowUpdateService` for definition sync | `./definitions` |
| `RUNTIME_STATUS_UPDATE_INTERVAL_SECONDS` | Heartbeat interval in dynamic mode (`0` disables heartbeat) | `30` |

## Local Development

### 1. Start dependencies

At minimum, start a reachable NATS instance at `NATS_URL`.

### 2. Configure environment

Create `.env` in the repository root (you can copy from `.env-example` and extend it).

### 3. Run Taurus

```bash
cargo run -p taurus
```

### 4. Run the execution suite

```bash
cargo run -p tests
```

This executes all JSON files in `./flows` and compares runtime outputs.

### 5. Run one flow manually

```bash
cargo run -p manual -- --path ./flows/01_return_object.json --index 0 --nats-url nats://127.0.0.1:4222
```

This is useful when debugging one case or remote-execution behavior.

## Testing

- Core unit/integration tests:

```bash
cargo test -p taurus-core
```

- Full workspace checks (recommended before merge):

```bash
cargo test
```

## Extending Taurus

### Add or modify built-in functions

- Implement handler logic in `crates/taurus-core/src/runtime/functions/*`
- Register function IDs via the `FUNCTIONS` arrays
- Registration is aggregated through `ALL_FUNCTION_SETS` in `runtime/functions/mod.rs`

### Remote execution routing rule

In the compiler, a node is treated as local when `definition_source` is:

- empty
- `taurus`
- prefixed with `draco`

Any other source is routed as remote execution to that service name.
