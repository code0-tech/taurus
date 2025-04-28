# std for PRIMITIVE: TEXT

## asBytes
Converts the text to a number array.

```json
{
  "runtime_name": "std::text::as_bytes",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER_ARRAY"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT_ARRAY"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "index"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "suffix"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "prefix"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "position"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "text"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "start"
    },
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "end"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "old"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "new"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "old"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "new"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "old"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "new"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "substring"
    }
  ],
  "return_type_identifier": "NUMBER"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "substring"
    }
  ],
  "return_type_identifier": "BOOLEAN"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "delimiter"
    }
  ],
  "return_type_identifier": "TEXT_ARRAY"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "prefix"
    }
  ],
  "return_type_identifier": "BOOLEAN"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "suffix"
    }
  ],
  "return_type_identifier": "BOOLEAN"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER_ARRAY"
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
      "data_type_identifier": "NUMBER_ARRAY",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "TEXT"
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
      "data_type_identifier": "TEXT",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "TEXT_ENCODING",
      "runtime_name": "encoding"
    }
  ],
  "return_type_identifier": "TEXT"
}
```

**Example**:

"hello", "base64" --> "aGVsbG8="
