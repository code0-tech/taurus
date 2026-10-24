# Test Execution Suite

This package runs JSON flow fixtures directly through Taurus's execution
engine and compares each result with the fixture's expected value. The
fixtures are split into two suites:

| Suite | Directory | Runs with |
| --- | --- | --- |
| `local` | `flows/local/` | No remote runtime. Flows that call an action fail as they would without one. |
| `remote` | `flows/remote/` | Simulated actions (`src/simulated_actions.rs`) standing in for real ones. |

## Run the execution suite

```console
cargo run --package taurus-tests
```

Pass `local` or `remote` to run a single suite:

```console
cargo run --package taurus-tests -- remote
```

The runner exits non-zero if any case fails.

## Add a flow

Put the fixture into the suite directory that matches what the flow does:

```json
{
	"name": "Descriptive snake case name",
	"description": "Description on what logic should be tested",
	"inputs": [
		{
			"input": "Input Value/Flow Input (JSON)",
				"expected_result": "Expected (JSON) Result of the flow"
        }
	],
	"flow": "A flow exported from Aquila using the protobuf JSON field representation"
}
```

## Simulated actions

Flows in the `remote` suite dispatch to these in-process action functions.
Point a node (or a function-backed sub-flow) at one with its
`definitionSource` and `runtimeFunctionId`:

| Service (`definitionSource`) | Function | Behavior |
| --- | --- | --- |
| `action.example` | `remote::identity(value)` | Returns `value`. |
| `action.example` | `remote::double(value)` | Returns `value * 2`. |
| `action.example` | `remote::fail()` | Always fails, to prove a function never runs. |
| `action.example` | `remote::sub_flow_function(function_identifier, definition_source)` | Returns a sub flow function instead of a value (`definition_source` may be `null` for Taurus functions). |
| `action.example` | `remote::apply(handler, value)` | Invokes the `handler` sub flow once with `value` and returns its result. |
| `action.example` | `remote::keep(handler)` | Receives a `handler` sub flow, never invokes it, and returns `"kept"`. |
| `action.websocket-action` | `for_each(list, consumer)` | Invokes the `consumer` sub flow once per element and returns `null`. |

To test new action behavior, add a function to `SimulatedActions` and
document it here.
