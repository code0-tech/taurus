# std for PRIMITIVE: TEXT

## asBytes
Converts the text to a number array.

```json
{
  "runtime_name": "std::text::as_bytes",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text to convert into bytes."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts the input text string into an array of byte values."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_mappers": [
    {
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "T"
    }
  ],
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "As Bytes"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts a text string into an array of byte values."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns an array of bytes representing the UTF-8 encoding of the given text."
    }
  ],
  "deprecation_message": [],
  "generic_keys": []
}
```

**Example**:

"hello" --> [104, 101, 108, 108, 111]

## byteSize
Returns the size of the text in bytes.

```json
{
  "runtime_name": "std::text::byte_size",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text whose byte size is to be calculated."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Calculates the size in bytes of the given text, typically its UTF-8 encoding length."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Byte Size"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the number of bytes required to encode the given text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the size in bytes of the provided text string, typically by counting UTF-8 encoded bytes."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> 5

## capitalize
Capitalizes the first character of the text.

```json
{
  "runtime_name": "std::text::capitalize",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to capitalize."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Capitalizes the first letter of the input text string."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Capitalize"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts the first character of the text to uppercase and leaves the rest unchanged."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string with the first letter capitalized."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> "Hello"

"world wide web" --> "World wide web"

## uppercase
Converts all characters in the text to uppercase.

```json
{
  "runtime_name": "std::text::uppercase",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to convert to uppercase."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts all characters in the input text string to uppercase."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Uppercase"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Transforms all letters in the text to their uppercase equivalents."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string with all characters converted to uppercase."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> "HELLO"

"Hello World" --> "HELLO WORLD"

## lowercase
Converts all characters in the text to lowercase.

```json
{
  "runtime_name": "std::text::lowercase",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to convert to lowercase."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Converts all characters in the input text string to lowercase."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Lowercase"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Transforms all letters in the text to their lowercase equivalents."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string with all characters converted to lowercase."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"HELLO" --> "hello"

"Hello World" --> "hello world"

## swapcase
Swaps the case of all characters in the text.

```json
{
  "runtime_name": "std::text::swapcase",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string whose case will be swapped."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Swaps the case of each letter in the input text: uppercase letters become lowercase, and vice versa."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Swap Case"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts uppercase letters to lowercase and lowercase letters to uppercase in the given text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string with the case of each character inverted."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"Hello" --> "hELLO"

"Hello World" --> "hELLO wORLD"

## chars
Splits the text into an array of characters.

```json
{
  "runtime_name": "std::text::chars",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to split into characters."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Splits the input text string into an array of its constituent characters."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "source": {
        "data_type_identifier": "TEXT"
      },
      "target": "R"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Characters"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns an array containing each character from the given text string."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Creates an array where each element is a single character from the original text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

**Example**:

"hello" --> ["h", "e", "l", "l", "o"]

## at
Returns the character at the specified index.

```json
{
  "runtime_name": "std::text::at",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string from which to extract the character."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text from which a character will be retrieved by index."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "index",
      "name": [
        {
          "code": "en-US",
          "content": "Index"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The zero-based position of the character to extract."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies which character to return from the text."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "TEXT"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Character at Index"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the character at the specified index in the text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Retrieves a single character from the input text based on the provided zero-based index."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello", 1 --> "e"

"world", 0 --> "w"

## trim
Removes whitespace from both ends of the text.

```json
{
  "runtime_name": "std::text::trim",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to trim whitespace from."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text from which leading and trailing whitespace characters will be removed."
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
      "content": "Trim Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Removes leading and trailing whitespace from the text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string with all leading and trailing whitespace characters removed from the input text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"  hello  " --> "hello"

"  hello world  " --> "hello world"

## apped
Concatenates two strings together.

```json
{
  "runtime_name": "std::text::append",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The initial text to which the suffix will be appended."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The base text string that will have another string appended to its end."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "suffix",
      "name": [
        {
          "code": "en-US",
          "content": "Suffix"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text to append to the original value."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The text string that will be concatenated to the end of the original text."
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
      "content": "Append Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Concatenates the suffix text to the end of the original text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string consisting of the original text followed by the specified suffix."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello", " world" --> "hello world"

"abc", "123" --> "abc123"

## prepend
Adds text to the beginning of the string.

```json
{
  "runtime_name": "std::text::prepend",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The initial text to which the prefix will be added."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The base text string that will have another string prepended to its beginning."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "prefix",
      "name": [
        {
          "code": "en-US",
          "content": "Prefix"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text to prepend before the original value."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The text string that will be concatenated to the start of the original text."
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
      "content": "Prepend Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Concatenates the prefix text to the beginning of the original text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new text string consisting of the specified prefix followed by the original text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"world", "hello " --> "hello world"

"123", "abc" --> "abc123"

## insert
Inserts text at the specified position.

```json
{
  "runtime_name": "std::text::insert",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The original text into which another text will be inserted."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the base string where the insertion happens."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "position",
      "name": [
        {
          "code": "en-US",
          "content": "Position"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The index at which the text will be inserted."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Zero-based index indicating where the new text should be inserted."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "text",
      "name": [
        {
          "code": "en-US",
          "content": "Text to Insert"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text that will be inserted into the original text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The substring to be inserted at the specified position."
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
      "content": "Insert Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Inserts a given text into the original text at the specified position."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string where the provided text is inserted at the zero-based position index within the original text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"helloworld", 5, " " --> "hello world"

"abcdef", 3, "123" --> "abc123def"

## length
Returns the length of the text.

```json
{
  "runtime_name": "std::text::length",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text whose length will be calculated."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Input string to determine the number of characters it contains."
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
      "content": "Length"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the number of characters in the given text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the length of the input string in terms of characters."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> 5

"hello world" --> 11

## remove
Removes a portion of text from the specified start index to end index.

```json
{
  "runtime_name": "std::text::remove",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The original text to remove a substring from."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text from which a substring will be removed."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "start",
      "name": [
        {
          "code": "en-US",
          "content": "Start Index"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The zero-based index where removal begins."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The starting position for removing characters from the text."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "end",
      "name": [
        {
          "code": "en-US",
          "content": "End Index"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The zero-based index where removal ends (exclusive)."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The position just after the last character to be removed."
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
      "content": "Remove Substring"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Removes the substring between the specified start and end indices from the input text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string with characters removed from start up to but not including end."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", 5, 6 --> "helloworld"

"abcdefg", 2, 5 --> "abfg"

## replace
Replaces all occurrences of a substring with another string.

```json
{
  "runtime_name": "std::text::replace",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text where replacements will be made."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the text in which all occurrences of the old substring will be replaced."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "old",
      "name": [
        {
          "code": "en-US",
          "content": "Old Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to be replaced."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "All occurrences of this substring in the original text will be replaced."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "new",
      "name": [
        {
          "code": "en-US",
          "content": "New Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to replace with."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This substring will replace each occurrence of the old substring."
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
      "content": "Replace Substring"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Replaces all occurrences of a specified substring with another substring in the input text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string where every instance of the old substring is replaced by the new substring."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", "world", "universe" --> "hello universe"

"ababab", "a", "c" --> "cbcbcb"

## replaceFirst
Replaces the first occurrence of a substring with another string.

```json
{
  "runtime_name": "std::text::replace_first",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text where the first replacement will be made."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This text contains the substring that will be replaced only once—the first occurrence."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "old",
      "name": [
        {
          "code": "en-US",
          "content": "Old Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to be replaced."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Only the first occurrence of this substring will be replaced in the original text."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "new",
      "name": [
        {
          "code": "en-US",
          "content": "New Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to replace with."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This substring will replace only the first occurrence of the old substring."
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
      "content": "Replace First Substring"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Replaces the first occurrence of a specified substring with another substring in the input text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string where only the first instance of the old substring is replaced by the new substring."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello hello", "hello", "hi" --> "hi hello"

"ababab", "a", "c" --> "cbabab"

## replaceLast
Replaces the last occurrence of a substring with another string.

```json
{
  "runtime_name": "std::text::replace_last",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Original Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text where the last replacement will be made."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This text contains the substring that will be replaced only once—the last occurrence."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "old",
      "name": [
        {
          "code": "en-US",
          "content": "Old Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to be replaced."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Only the last occurrence of this substring will be replaced in the original text."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "new",
      "name": [
        {
          "code": "en-US",
          "content": "New Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to replace with."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This substring will replace only the last occurrence of the old substring."
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
      "content": "Replace Last Substring"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Replaces the last occurrence of a specified substring with another substring in the input text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string where only the last instance of the old substring is replaced by the new substring."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello hello", "hello", "hi" --> "hello hi"

"ababab", "a", "c" --> "ababcb"

## hex
Converts the text to a hexadecimal representation.

```json
{
  "runtime_name": "std::text::hex",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Input Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to be converted to its hexadecimal representation."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This function converts each character of the input text into its corresponding hexadecimal code, returning the concatenated hex string."
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
      "content": "Text to Hexadecimal"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts a text string into a hexadecimal representation."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a string containing the hexadecimal values corresponding to each character of the input text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> "68656c6c6f"

"ABC" --> "414243"

## octal
Converts the text to an octal representation.

```json
{
  "runtime_name": "std::text::octal",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Input Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to be converted to its octal representation."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This function converts each character of the input text into its corresponding octal code, returning the concatenated octal string."
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
      "content": "Text to Octal"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts a text string into an octal representation."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a string containing the octal values corresponding to each character of the input text."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello" --> "150145154154157"

"ABC" --> "101102103"

## indexOf
Returns the index of the first occurrence of a substring, or -1 if not found.

```json
{
  "runtime_name": "std::text::index_of",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to search within."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The text string to search within."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "substring",
      "name": [
        {
          "code": "en-US",
          "content": "Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The substring to find inside the text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The substring to find inside the text."
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
      "content": "Index Of"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the first occurrence index of the substring within the text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the zero-based index of the first occurrence of the substring in the text. Returns -1 if the substring is not found."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", "world" --> 6

"hello world", "not found" --> -1

## contains
Checks if the text contains a specified substring.

```json
{
  "runtime_name": "std::text::contains",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The main text to search within."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The main text to search within."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "substring",
      "name": [
        {
          "code": "en-US",
          "content": "Substring"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text to search for inside the main text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The text to search for inside the main text."
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
      "content": "Contains"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks if the main text contains the specified substring."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the substring is found anywhere in the main text; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "error_type_identifiers": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", "world" --> true

"hello world", "not found" --> false

## split
Splits the text into an array of strings based on a delimiter.

```json
{
  "runtime_name": "std::text::split",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text to be split."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text to be split."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "delimiter",
      "name": [
        {
          "code": "en-US",
          "content": "Delimiter"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The delimiter string to split the text by."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The delimiter string to split the text by."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "source": {
        "data_type_identifier": "TEXT"
      },
      "target": "R"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Split"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Splits the input text into an array of substrings based on the specified delimiter."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns an array of substrings obtained by splitting the input text at each occurrence of the delimiter."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

**Example**:

"hello world", " " --> ["hello", "world"]

"a,b,c", "," --> ["a", "b", "c"]

## reverse
Reverses the text.

```json
{
  "runtime_name": "std::text::reverse",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text to be reversed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text to be reversed."
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
      "content": "Reverse"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Reverses the characters in the input text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new string with the characters of the input text in reverse order."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": [],
  "error_type_identifiers": []
}
```

**Example**:

"hello" --> "olleh"

"world" --> "dlrow"

## startWith
Checks if the text starts with a specified prefix.

```json
{
  "runtime_name": "std::text::start_with",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text to check."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text to check."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "prefix",
      "name": [
        {
          "code": "en-US",
          "content": "Prefix"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The prefix to test against the input text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The prefix to test against the input text."
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
      "content": "Starts With"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks if the input text starts with the specified prefix."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the input text begins with the given prefix; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", "hello" --> true

"hello world", "world" --> false

## endsWith
Checks if the text ends with a specified suffix.

```json
{
  "runtime_name": "std::text::ends_with",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input text to check."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input text to check."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "suffix",
      "name": [
        {
          "code": "en-US",
          "content": "Suffix"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The suffix to test against the input text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The suffix to test against the input text."
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
      "content": "Ends With"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks if the input text ends with the specified suffix."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the input text ends with the given suffix; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"hello world", "world" --> true

"hello world", "hello" --> false

## toASCII
Converts each character to its ASCII code as an array of numbers.

```json
{
  "runtime_name": "std::text::to_ascii",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "Input text to convert to ASCII codes."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Input text to convert to ASCII codes."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "name": [
    {
      "code": "en-US",
      "content": "To ASCII"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts each character of the input text into its corresponding ASCII numerical code."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns an array of numbers where each number represents the ASCII code of the corresponding character in the input text."
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "R"
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

**Example**:

"hello" --> [104, 101, 108, 108, 111]

"ABC" --> [65, 66, 67]

## fromASCII
Converts an array of ASCII codes to a text string.

```json
{
  "runtime_name": "std::text::from_ascii",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "ASCII Codes"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "Array of ASCII numeric codes representing characters."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Array of ASCII numeric codes representing characters."
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
      "content": "From ASCII"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts an array of ASCII codes back into the corresponding text string."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Takes an array of numbers where each number is an ASCII code, and returns the string they represent."
    }
  ],
  "generic_mappers": [
    {
      "parameter_id": "value",
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "T"
    }
  ],
  "generic_keys": [],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

**Example**:

[104, 101, 108, 108, 111] --> "hello"

[65, 66, 67] --> "ABC"

## encode
Encodes the text using a specified encoding.

```json
{
  "runtime_name": "std::text::encode",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "value",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to encode."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The text string to encode."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT_ENCODING"
      },
      "runtime_name": "encoding",
      "name": [
        {
          "code": "en-US",
          "content": "Encoding Type"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The encoding scheme to apply (e.g., UTF-8, Base64)."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The encoding scheme to apply (e.g., UTF-8, Base64)."
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
      "content": "Encode Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Encodes the input text into the specified encoding format."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Transforms the given text string into a representation encoded by the specified encoding scheme."
    }
  ],
  "generic_keys": [],
  "generic_mappers": [],
  "error_type_identifiers": [],
  "deprecation_message": []
}
```

**Example**:

"hello", "base64" --> "aGVsbG8="

## isEqual

```json
{
  "runtime_name": "std::text::is_equal",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first text string to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The first input text for equality comparison."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second text string to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The second input text for equality comparison."
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
      "content": "Checks whether the two input text strings are equal."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Determines if the two given text inputs are exactly the same, returning true if equal, false otherwise."
    }
  ],
  "error_type_identifiers": [],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"TEXT", "TEXT" --> true

"TEXT", "WORD" --> false
