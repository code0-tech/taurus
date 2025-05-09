# std for PRIMITIVE: BOOLEAN

## asNumber
Will convert the boolean to a number.

```json
{
  "runtime_name": "std::boolean::as_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "BOOLEAN",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER"
}
```

**Example**:

false --> 0

true --> 1

## asText
Will convert the boolean to a string.

```json
{
  "runtime_name": "std::boolean::as_text",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "BOOLEAN",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
}
```

**Example**:

false --> "false"

true --> "true"


## fromNumber
Will convert the number to a boolean.

```json
{
  "runtime_name": "std::boolean::from_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "BOOLEAN"
}
```

**Example**:

0 --> false

1 --> true


## fromText
Will convert the string to a boolean.

```json
{
  "runtime_name": "std::boolean::from_text",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "BOOLEAN"
}
```

**Example**:

"false" --> false

"true" --> true


## isEqual
Will compare one boolean to another.

```json
{
  "runtime_name": "std::boolean::isEqual",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "BOOLEAN",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "BOOLEAN",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "BOOLEAN"
}
```

**Example**:

false, false --> true

true, false --> false
