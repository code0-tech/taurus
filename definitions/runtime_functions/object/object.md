## remove
```json
{
  "runtime_name": "std::object::remove",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "OBJECT"
      },
      "runtime_name": "object"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "key"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "OBJECT"
  }
}
```

## containsKey
```json
{
  "runtime_name": "std::object::contains_key",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "OBJECT"
      },
      "runtime_name": "object"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "key"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  }
}
```


## keys
```json
{
  "runtime_name": "std::object::keys",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "OBJECT"
      },
      "runtime_name": "object"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_mappers": [
    {
      "source": "TEXT",
      "target": "T"
    }
  ]
}
```

## size
```json
{
  "runtime_name": "std::object::size",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "OBJECT"
      },
      "runtime_name": "object"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  }
}
```

## set
```json
{
  "runtime_name": "std::object::set",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "OBJECT"
      },
      "runtime_name": "object"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "key"
    },
    {
      "data_type_identifier": {
        "generic_key": "I"
      },
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "OBJECT"
  },
  "generic_keys": ["I"],
  "generic_mappers": [
    {
      "paramter_id": "value"
      "source": "I",
      "target": "I"
    }
  ]
}
```
