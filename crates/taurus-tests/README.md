# Test Execution Suite

This package runs every JSON fixture in `./flows` directly through Taurus's
execution engine and compares the result with the fixture's expected value.

## Run the execution suite

```console
cargo run --package taurus-tests
```

## Add a flow

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

## Testing remote function sub-flows

Add a `remote` fixture to the case when the flow should dispatch a
function-backed sub-flow without connecting to NATS:

```json
"remote": {
  "targetService": "example",
  "functionIdentifier": "remote::identity",
  "resultParameter": "value"
}
```

The fixture validates the target service and function identifier, then returns
the named request parameter as the remote result. This lets a flow verify both
remote routing and sub-flow setting materialization.
