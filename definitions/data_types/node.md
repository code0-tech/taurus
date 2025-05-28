## PREDICATE
```json
{
  "identifier": "PREDICATE",
  "variant": 7,
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
  "generic_keys": ["T"],
  "name": [
    {
      "code": "en-US",
      "content": "Predicate"
    }
  ]
}
```

## CONSUMER
```json
{
  "identifier": "CONSUMER",
  "variant": 7,
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
  "generic_keys": ["T"],
  "name": [
    {
      "code": "en-US",
      "content": "Consumer"
    }
  ]

}
```

## TRANSFORM
```json
{
  "identifier": "TRANSFORM",
  "variant": 7,
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
  "generic_keys": ["I", "R"],
  "name": [
    {
      "code": "en-US",
      "content": "Transform"
    }
  ]

}
```

## COMPARITOR
```json
{
  "identifier": "COMPARITOR",
  "variant": 7,
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
  "generic_keys": ["I"],
  "name": [
    {
      "code": "en-US",
      "content": "Comparitor"
    }
  ]
}
```
