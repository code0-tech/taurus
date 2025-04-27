# std for PRIMITIVE: BOOLEAN

## asNumber
Will convert the boolean to a number.

```json
{
  "runtime_name": "std::boolean::as_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

"false" --> false

"true" --> true


## negate
Will negate the boolean.

```json
{
  "runtime_name": "std::boolean::negate",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

false --> true

true --> false
