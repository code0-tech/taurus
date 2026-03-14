# Test Execution Suite

This is the service for running a full execution for example flows. 

## How to run the execution suite?

```console
cargo run --package tests
```

## How to add a flow to the suite?

```json
{
	"name": "Descriptive snake case name",
	"description": "Description on what logic should be tested",
	"inputs": [
		{
			"input": "Input Value/Flow Input (JSON)",
			"expected_result": "Expected (JSON) Result of the flow",
        }
	],
	"flow": "The flow object (Exported from Aquila), parsed from Protobuf Message struct so it should be protobuf value not json value"
}

```

An example

```json
{
	"name": "01_return_object",
	"description": "This flow expects a simple http response object as the return value",
	"inputs": [
		{
			"input": null,
			"expected_result": {
				"status_code": 200,
				"headers": {
					"Header": "X"
				},
				"payload": "Hello World"
			}
		}
	],
	"flow": {
		"starting_node_id": "1",
		"node_functions": [
			{
				"databaseId": "2",
				"runtimeFunctionId": "rest::control::respond",
				"parameters": [
					{
						"databaseId": "4",
						"runtimeParameterId": "http_response",
						"value": {
							"referenceValue": {
								"nodeId": "1"
							}
						}
					}
				]
			},
			{
				"databaseId": "1",
				"runtimeFunctionId": "http::response::create",
				"parameters": [
					{
						"databaseId": "1",
						"runtimeParameterId": "http_status_code",
						"value": {
							"literalValue": {
								"stringValue": "200"
							}
						}
					},
					{
						"databaseId": "2",
						"runtimeParameterId": "headers",
						"value": {
							"literalValue": {
								"structValue": {
									"fields": {
										"Header": {
											"stringValue": "X"
										}
									}
								}
							}
						}
					},
					{
						"databaseId": "3",
						"runtimeParameterId": "payload",
						"value": {
							"literalValue": {
								"stringValue": "Hello World"
							}
						}
					}
				],
				"nextNodeId": "2"
			}
		]
	}
}
```
