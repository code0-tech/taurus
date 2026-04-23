# Taurus Runtime Error Codes

This document is the canonical catalog for runtime error codes emitted by Taurus runtime crates (`taurus-core` and `taurus-provider`).

## Code Format

- `T-STD-XXXXX`: Errors originating inside standard function implementations under `runtime/functions/*`.
- `T-CORE-XXXXXX`: Errors originating from core runtime infrastructure (`engine`, `handler`, type conversion, app-layer mapping).
- `T-PROV-XXXXXX`: Errors originating from provider integrations (transport adapters, remote runtime connectors).

## Code Table

| Code | Layer | Description | Typical Trigger | Primary Source |
| --- | --- | --- | --- | --- |
| `T-STD-00001` | Standard Functions | A standard runtime function failed due to invalid input shape/type, unsupported value semantics, or function-specific runtime constraints. | Wrong argument type, invalid value conversion, out-of-range operation, malformed function input. | `runtime/functions/*` |
| `T-STD-00002` | Standard Functions | Key of object was not present. | Field is not present, key generally does not exists inside object. | `runtime/functions/object.rs` |
| `T-CORE-000001` | Engine | Requested node id does not exist in the compiled flow plan. | Thunk/reference points to a node id not present in `CompiledFlow`. | `runtime/engine/executor.rs` |
| `T-CORE-000002` | Engine | Handler registry has no implementation for the node's runtime function id. | Function id was not registered in `FunctionStore`. | `runtime/engine/executor.rs` |
| `T-CORE-000003` | Engine | Flow requires remote execution but no remote runtime adapter was configured. | Node execution target is remote while `RemoteRuntime` is `None`. | `runtime/engine/executor.rs` |
| `T-CORE-000004` | Engine | Reference lookup failed in the execution value store. | Missing prior node result, missing flow input path, or unresolved input reference. | `runtime/engine/executor.rs` |
| `T-CORE-000005` | Engine | Remote request cannot be assembled because parameter metadata and resolved values diverge. | Parameter count mismatch during remote request materialization. | `runtime/engine/executor.rs` |
| `T-CORE-000101` | Compiler | Flow compilation failed because a node id appears more than once. | Duplicate `database_id` in input nodes. | `runtime/engine/compiler.rs` |
| `T-CORE-000102` | Compiler | Flow compilation failed because the declared start node is absent. | `start_node_id` not found in node list. | `runtime/engine/compiler.rs` |
| `T-CORE-000103` | Compiler | Flow compilation failed because a `next` edge points to a missing node. | `next_node_id` references unknown node id. | `runtime/engine/compiler.rs` |
| `T-CORE-000104` | Compiler | Flow compilation failed because a parameter is structurally incomplete. | Parameter has no value payload in IR. | `runtime/engine/compiler.rs` |
| `T-CORE-000201` | Handler | Handler argument arity contract was violated before function execution began. | `args!`/`no_args!` macro expected different argument count. | `handler/macros.rs` |
| `T-CORE-000202` | Handler | Handler argument type conversion failed during typed extraction. | `TryFromArgument` expected type does not match provided argument. | `handler/argument.rs` |
| `T-CORE-000301` | App Error Mapping | Application configuration failure mapped into runtime error format. | Invalid/missing runtime config surfaced as `Error::Configuration`. | `types/errors/error.rs` |
| `T-CORE-000302` | App Error Mapping | Invalid application state mapped into runtime error format. | Illegal lifecycle/state transition surfaced as `Error::State`. | `types/errors/error.rs` |
| `T-CORE-000303` | App Error Mapping | Transport/dependency communication failure mapped into runtime error format. | Network/broker/downstream call failure surfaced as `Error::Transport`. | `types/errors/error.rs` |
| `T-CORE-000304` | App Error Mapping | Serialization/deserialization failure mapped into runtime error format. | Encoding/decoding/parsing failure surfaced as `Error::Serialization`. | `types/errors/error.rs` |
| `T-CORE-000399` | App Error Mapping | Internal application failure mapped into runtime error format. | Catch-all non-domain internal failure surfaced as `Error::Internal`. | `types/errors/error.rs` |
| `T-CORE-999999` | Runtime Error Fallback | Default fallback runtime error code when no explicit mapping is provided. | `RuntimeError::default()` used as defensive fallback. | `types/errors/runtime_error.rs` |
| `T-PROV-000001` | Provider Remote Runtime | Remote request to NATS did not yield a valid response message. | NATS request failed or timed out while waiting for remote runtime answer. | `taurus-provider/providers/remote/nats_remote_runtime.rs` |
| `T-PROV-000002` | Provider Remote Runtime | Remote runtime response could not be decoded into expected protobuf structure. | Received payload is malformed, truncated, or schema-incompatible for `ExecutionResult`. | `taurus-provider/providers/remote/nats_remote_runtime.rs` |
| `T-PROV-000003` | Provider Remote Runtime | Remote runtime response decoded, but contained no concrete result field. | `ExecutionResult` exists but `result` is `None` (protocol contract violation). | `taurus-provider/providers/remote/nats_remote_runtime.rs` |

## Provider Note

`taurus-provider` can also forward remote service errors with service-owned codes (for example codes returned inside Aquila `ExecutionResult::Error`). Those are intentionally preserved instead of remapped, so they are not enumerated as static Taurus provider codes here.
