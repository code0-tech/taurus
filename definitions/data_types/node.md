```json
{
  "identifier": "PREDICATE",
  "variant": "NODE",
  "rules": [
    {
      "return_type": {
        "data_type_identifier": "BOOLEAN"
      }
    },
    {
      "input_type": [
        {
          "data_type_identifier": {
            "generic_key": "T"
          },
          "input_identifier": "predicate"
        }
      ]
    }
  ],
  "generic_keys": ["T"]
}
```

```json
{
  "identifier": "CONSUMER",
  "variant": "NODE",
  "rules": [
    {
      "input_type": [
        {
          "data_type_identifier": {
            "generic_key": "T"
          },
          "input_identifier": "consumer"
        }
      ]
    }
  ],
  "generic_keys": ["T"]
}
```

```json
{
  "identifier": "TRANSFORM",
  "variant": "NODE",
  "rules": [
    {
      "return_type": {
        "data_type_identifier": {
          "generic_key": "R"
        }
      }
    },
    {
      "input_type": [
        {
          "data_type_identifier": {
            "generic_key": "I"
          },
          "input_identifier": "transform"
        }
      ]
    }
  ],
  "generic_keys": ["I", "R"]
}
```

```json
{
  "identifier": "COMPARITOR",
  "variant": "NODE",
  "rules": [
    {
      "return_type": {
        "data_type_identifier": {
          "data_type_identifier": "NUMBER"
        }
      }
    },
    {
      "input_type": [
        {
          "data_type_identifier": {
            "generic_key": "I"
          },
          "input_identifier": "left"
        },
        {
          "data_type_identifier": {
            "generic_key": "I"
          },
          "input_identifier": "right"
        }
      ]
    }
  ],
  "generic_keys": ["I"]
}
```
