# std for PRIMITIVE: BOOLEAN

## asNumber
Will convert the boolean to a number.

```json
{
  "runtime_name": "std::boolean::as_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "BOOLEAN"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The boolean value to convert."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts a boolean value to a number."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "name": [
    {
      "code": "en-US",
      "content": "As Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Will convert the boolean to a number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Converts a boolean value to a number."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
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
      "data_type_identifier": {
        "data_type_identifier": "BOOLEAN"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The boolean value to convert."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts a boolean value to a text string."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "name": [
    {
      "code": "en-US",
      "content": "As Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Will convert the boolean to text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Converts a boolean value to a text string."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
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
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to convert."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts a number to a boolean value."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "name": [
    {
      "code": "en-US",
      "content": "From Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Will convert the number to a boolean."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Converts a number to a boolean value. Typically, 0 maps to false and non-zero maps to true."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
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
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to convert."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts a text string to a boolean value."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "name": [
    {
      "code": "en-US",
      "content": "From Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Will convert the string to a boolean."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Converts a text string to a boolean value. Recognizes 'true' and 'false' (case-insensitive)."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
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
      "data_type_identifier": {
        "data_type_identifier": "BOOLEAN"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first boolean value to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The first input to compare."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "BOOLEAN"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second boolean value to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The second input to compare."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "name": [
    {
      "code": "en-US",
      "content": "Is Equal"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Will check if the two booleans are equal."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Compares two boolean values for equality. Returns true if they are the same, false otherwise."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

false, false --> true

true, false --> false
