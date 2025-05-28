# std for PRIMITIVE: NUMBER

## add
Adds two numbers together.

```json
{
  "runtime_name": "std::number::add",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first number to add."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Adds two numbers together."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second number to add."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Adds two numbers together."
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
      "content": "Adds two numbers together."
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Adds two numbers together."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Adds two numbers together."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3 + 2 --> 5

## multiply
Multiplies two numbers together.

```json
{
  "runtime_name": "std::number::multiply",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first number to multiply."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies the first operand in the multiplication operation."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second number to multiply."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies the second operand in the multiplication operation."
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
      "content": "Multiply"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Multiplies two numbers together."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Takes two numeric inputs and returns their product."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3 * 2 --> 6

## subtract
Subtracts the second number from the first.

```json
{
  "runtime_name": "std::number::subtract",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "Minuend"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number from which another number (the subtrahend) is to be subtracted."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that will have the second value subtracted from it."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Subtrahend"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to subtract from the first number (the minuend)."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the value that will be subtracted from the first number."
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
      "content": "Subtract"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Subtracts the second number from the first number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the result of subtracting the second numeric input from the first."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5 - 2 --> 3

## divide
Divides the first number by the second.

```json
{
  "runtime_name": "std::number::divide",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "Dividend"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to be divided."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numerator or the number that will be divided by the second value."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Divisor"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number by which to divide the first number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the denominator or the value that divides the first number."
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
      "content": "Divide"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Divides the first number by the second number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the result of dividing the first numeric input (dividend) by the second (divisor)."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

6 / 2 --> 3

## modulo
Returns the remainder after division of the first number by the second.

```json
{
  "runtime_name": "std::number::modulo",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "Dividend"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to be divided to find the remainder."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that will be divided by the second value to calculate the remainder."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Divisor"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number by which the first number is divided to get the remainder."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number used to divide the dividend and obtain the remainder."
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
      "content": "Modulo"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the remainder after dividing the first number by the second."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the modulus (remainder) of dividing the first numeric input by the second."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

7 % 3 --> 1

## abs
Converts a number to its absolute value.

```json
{
  "runtime_name": "std::number::abs",
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
          "content": "The number for which to compute the absolute value."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numeric input. The result will be its absolute (non-negative) value."
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
      "content": "Absolute Value"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the absolute value of a number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Removes the sign from the input number, returning its non-negative value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

-5 --> 5

## isPositive
Checks if a number is positive.

```json
{
  "runtime_name": "std::number::is_positive",
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
          "content": "The number to check for positivity."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numeric input that will be evaluated to determine whether it is greater than zero."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Is Positive"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks whether a number is greater than zero."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Evaluates the input number and returns true if it is positive (greater than zero), otherwise false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5 --> true
-2 --> false

## isGreater
Checks if the first number is greater than the second.

```json
{
  "runtime_name": "std::number::is_greater",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to compare against the second number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that will be evaluated to determine if it is greater than the second number."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to compare with the first number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that the first number will be compared to."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Is Greater"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks whether the first number is greater than the second number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the first numeric input is greater than the second; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5 > 3 --> true
2 > 4 --> false

## isLess
Checks if the first number is less than the second.

```json
{
  "runtime_name": "std::number::is_less",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to compare with the second number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that will be evaluated to determine if it is less than the second number."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to compare against the first number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the number that the first number will be compared to."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Is Less"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks whether the first number is less than the second number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the first numeric input is less than the second; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3 < 5 --> true
4 < 2 --> false

## isZero
Checks if a number is zero.

```json
{
  "runtime_name": "std::number::is_zero",
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
          "content": "The number to check if it is zero."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numeric input evaluated to determine whether it equals zero."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Is Zero"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks whether the given number is exactly zero."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the input number is zero; otherwise, returns false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> true
5 --> false

## square
Multiplies a number by itself.

```json
{
  "runtime_name": "std::number::square",
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
          "content": "The number to be squared."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numeric input that will be multiplied by itself."
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
      "content": "Square"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the square of the given number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the value multiplied by itself, effectively raising it to the power of 2."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

4 --> 16

## exponential
Raises a number to the specified power.

```json
{
  "runtime_name": "std::number::exponential",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "base",
      "name": [
        {
          "code": "en-US",
          "content": "Base"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The base number to be raised to a power."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the numeric value that will be raised to the power of the exponent."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "exponent",
      "name": [
        {
          "code": "en-US",
          "content": "Exponent"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The exponent to raise the base number by."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This numeric value indicates the power to which the base is raised."
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
      "content": "Exponential"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Raises a base number to the power of an exponent."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the result of raising the base to the power specified by the exponent."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

2^3 --> 8

## PI
Returns the mathematical constant Pi.

```json
{
  "runtime_name": "std::number::pi",
  "runtime_parameter_definitions": [],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Pi"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the mathematical constant π (pi)."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Provides the constant value of pi, approximately 3.14159, used in many mathematical calculations."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

PI --> 3.14159265359...

## EULER
Returns the mathematical constant e (Euler's number).

```json
{
  "runtime_name": "std::number::euler",
  "runtime_parameter_definitions": [],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Euler's Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the mathematical constant e (Euler's number)."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Provides the constant value of Euler's number, approximately 2.71828, which is the base of the natural logarithm."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

EULER --> 2.71828182846...

## INFINITY
Returns the representation of infinity.

```json
{
  "runtime_name": "std::number::infinity",
  "runtime_parameter_definitions": [],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Infinity"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the mathematical concept of positive infinity."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Provides the representation of positive infinity, used to represent an unbounded value in computations."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

INFINITY --> ∞

## roundUp
Rounds a number up to the nearest integer.

```json
{
  "runtime_name": "std::number::round_up",
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
          "content": "The number to be rounded up."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input that will be rounded upwards."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "decimals",
      "name": [
        {
          "code": "en-US",
          "content": "Decimal Places"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number of decimal places to round up to."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies how many decimal digits to keep after rounding up."
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
      "content": "Round Up"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Rounds a number upward to the specified number of decimal places."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Performs rounding on the given value, always rounding up to the nearest value at the given decimal precision."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3.2 (0) --> 4
3.8 (0) --> 4
3.008 (3) --> 3.009
3.008 (1) --> 3.1

## roundDown
Rounds a number down to the nearest integer.

```json
{
  "runtime_name": "std::number::round_down",
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
          "content": "The number to be rounded down."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input that will be rounded downwards."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "decimals",
      "name": [
        {
          "code": "en-US",
          "content": "Decimal Places"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number of decimal places to round down to."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies how many decimal digits to keep after rounding down."
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
      "content": "Round Down"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Rounds a number downward to the specified number of decimal places."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Performs rounding on the given value, always rounding down to the nearest value at the given decimal precision."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3.2 (0) --> 3
3.8 (0) --> 3

3.002 (1) --> 3
3.778 (2) --> 3.7

## round
Rounds a number to the nearest integer.

```json
{
  "runtime_name": "std::number::round",
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
          "content": "The number to be rounded."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input that will be rounded to the nearest value."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "decimals",
      "name": [
        {
          "code": "en-US",
          "content": "Decimal Places"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number of decimal places to round to."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies how many decimal digits to keep after rounding."
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
      "content": "Round"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Rounds a number to the nearest value at the specified number of decimal places."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Performs standard rounding on the given value, rounding up or down depending on the fractional component."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

3.2 (0) --> 3
3.8 (0) --> 4

3.002 (1) --> 3
3.778 (2) --> 3.78

## squareRoot
Calculates the square root of a number.

```json
{
  "runtime_name": "std::number::square_root",
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
          "content": "The number to find the square root of."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input for which the square root will be calculated."
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
      "content": "Square Root"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the square root of the given number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the positive square root of the input number."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

16 --> 4

## root
Calculates the nth root of a number.

```json
{
  "runtime_name": "std::number::root",
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
          "content": "The number from which the root will be extracted."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input for which the root will be calculated."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "root_exponent",
      "name": [
        {
          "code": "en-US",
          "content": "Root Exponent"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The degree of the root to extract."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies which root to calculate (e.g., 2 for square root, 3 for cube root)."
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
      "content": "Root"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the root of a number given a root exponent."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the nth root of the input number, where n is specified by the root exponent."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

27, 3 --> 3 (cube root of 27)

## log
Calculates the logarithm of a number with the specified base.

```json
{
  "runtime_name": "std::number::log",
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
          "content": "The number to compute the logarithm for."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input whose logarithm is to be calculated."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "base",
      "name": [
        {
          "code": "en-US",
          "content": "Base"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The base of the logarithm."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies the logarithmic base to use (e.g., 10 for common log, e for natural log)."
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
      "content": "Logarithm"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Calculates the logarithm of a number with respect to a specified base."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the logarithm of the given value using the specified base."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

100, 10 --> 2 (log base 10 of 100)

## ln
Calculates the natural logarithm (base e) of a number.

```json
{
  "runtime_name": "std::number::ln",
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
          "content": "The number to compute the natural logarithm for."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input whose natural logarithm (log base e) will be calculated."
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
      "content": "Natural Logarithm"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Calculates the natural logarithm (log base e) of a number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the natural logarithm of the given value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

2.71828... --> 1 (ln of e)

## fromText
Converts a string to a number.

```json
{
  "runtime_name": "std::number::from_text",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "text",
      "name": [
        {
          "code": "en-US",
          "content": "Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The text string to convert to a number."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Parses the input text and attempts to convert it to a numeric value."
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
      "content": "Number from Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts a text string into a number if possible."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Attempts to parse the provided text input and return its numeric equivalent."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

"123" --> 123
"3.14" --> 3.14

## asText
Converts a number to a text.

```json
{
  "runtime_name": "std::number::as_text",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "number",
      "name": [
        {
          "code": "en-US",
          "content": "Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The number to convert to text."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input that will be converted to its text representation."
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
      "content": "Number as Text"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Converts a number into its textual representation."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Transforms the given numeric value into a string format."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

123 --> "123"
3.14 --> "3.14"


## min
Returns the smaller of two numbers.

```json
{
  "runtime_name": "std::number::min",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "One of the two numbers for which the minimum value will be determined."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The other number involved in the minimum value comparison."
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
      "content": "Minimum"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the smaller of two numbers."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Compares two numbers and returns the minimum value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5, 3 --> 3
-1, 2 --> -1

## max
Returns the larger of two numbers.

```json
{
  "runtime_name": "std::number::max",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "One of the two numbers for which the maximum value will be determined."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The other number involved in the maximum value comparison."
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
      "content": "Maximum"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the larger of two numbers."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Compares two numbers and returns the maximum value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5, 3 --> 5
-1, 2 --> 2

## negate
Returns the additive inverse of a number.

```json
{
  "runtime_name": "std::number::negate",
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
          "content": "The number to negate."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The numeric input whose sign will be inverted."
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
      "content": "Negate"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the negation of a number (multiplies by -1)."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the additive inverse of the given number."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

5 --> -5
-3 --> 3

## randomNumber
Generates a random number between the specified minimum and maximum values.

```json
{
  "runtime_name": "std::number::random_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "min",
      "name": [
        {
          "code": "en-US",
          "content": "Minimum"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The minimum value in the random number range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Defines the lower bound (inclusive) for the random number generation."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "max",
      "name": [
        {
          "code": "en-US",
          "content": "Maximum"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The maximum value in the random number range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Defines the upper bound (inclusive) for the random number generation."
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
      "content": "Random Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Generates a random number between the specified minimum and maximum values."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a randomly generated number within the given range, inclusive of both minimum and maximum."
    }
  ],
  "error_type_identifiers": [],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

1, 6 --> 3 (random number between 1 and 6, inclusive)

## sin
Calculates the sine of an angle (in radians).

```json
{
  "runtime_name": "std::number::random_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "min",
      "name": [
        {
          "code": "en-US",
          "content": "Minimum Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The lower bound of the random number range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Defines the minimum value (inclusive) for the random number generation."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "max",
      "name": [
        {
          "code": "en-US",
          "content": "Maximum Value"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The upper bound of the random number range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Defines the maximum value (inclusive) for the random number generation."
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
      "content": "Random Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Generates a random number within the specified range."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a pseudo-random number between the given minimum and maximum values."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 0
1.5708 (π/2) --> 1

## cos
Calculates the cosine of an angle (in radians).

```json
{
  "runtime_name": "std::number::cos",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "radians",
      "name": [
        {
          "code": "en-US",
          "content": "Radians"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The angle in radians to compute the cosine of."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Computes the cosine of the given angle in radians."
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
      "content": "Cosine"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the cosine of the specified angle in radians."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the cosine value of the input angle measured in radians."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 1
3.14159 (π) --> -1

## tan
Calculates the tangent of an angle (in radians).

```json
{
  "runtime_name": "std::number::tan",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "radians",
      "name": [
        {
          "code": "en-US",
          "content": "Radians"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The angle in radians to compute the tangent of."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Computes the tangent of the given angle in radians."
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
      "content": "Tangent"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the tangent of the specified angle in radians."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the tangent value of the input angle measured in radians."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 0
0.7854 (π/4) --> 1

## arcsin
Calculates the inverse sine (in radians).

```json
{
  "runtime_name": "std::number::arcsin",
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
          "content": "The number representing the sine value, must be between -1 and 1."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Calculates the arcsine (inverse sine) of the input value."
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
      "content": "Arcsine"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the arcsine of a number, in radians."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the angle in radians whose sine is the given number."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 0
1 --> 1.5708 (π/2)

## arccos
Calculates the inverse cosine (in radians).

```json
{
  "runtime_name": "std::number::arccos",
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
          "content": "The number representing the cosine value, must be between -1 and 1."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Calculates the arccosine (inverse cosine) of the input value."
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
      "content": "Arccosine"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the arccosine of a number, in radians."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the angle in radians whose cosine is the given number."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

1 --> 0
-1 --> 3.14159 (π)

## arctan
Calculates the inverse tangent (in radians).

```json
{
  "runtime_name": "std::number::arctan",
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
          "content": "The number representing the tangent value."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Calculates the arctangent (inverse tangent) of the input value."
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
      "content": "Arctangent"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the arctangent of a number, in radians."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Computes the angle in radians whose tangent is the given number."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 0
1 --> 0.7854 (π/4)

## sinh
Calculates the hyperbolic sine of a number.

```json
{
  "runtime_name": "std::number::sinh",
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
          "content": "The number for which to calculate the hyperbolic sine."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Computes the hyperbolic sine of the given number."
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
      "content": "Hyperbolic Sine"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the hyperbolic sine of a number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the hyperbolic sine (sinh) of the input value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 0
1 --> 1.1752

## cosh
Calculates the hyperbolic cosine of a number.

```json
{
  "runtime_name": "std::number::cosh",
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
          "content": "The number for which to calculate the hyperbolic cosine."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Computes the hyperbolic cosine of the given number."
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
      "content": "Hyperbolic Cosine"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the hyperbolic cosine of a number."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Calculates the hyperbolic cosine (cosh) of the input value."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

0 --> 1
1 --> 1.5431

## clamp
Constrains a number to be within a specified range.

```json
{
  "runtime_name": "std::number::clamp",
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
          "content": "The number to be clamped within the range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The input number that will be limited to the specified range."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "min",
      "name": [
        {
          "code": "en-US",
          "content": "Minimum"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The lower bound of the clamping range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The minimum allowed value in the clamping operation."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "max",
      "name": [
        {
          "code": "en-US",
          "content": "Maximum"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The upper bound of the clamping range."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The maximum allowed value in the clamping operation."
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
      "content": "Clamp"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Limits a number to be within a specified range."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the given number clamped between the minimum and maximum bounds."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

10, 0, 5 --> 5 (clamped to max)
-3, 0, 5 --> 0 (clamped to min)
3, 0, 5 --> 3 (within range, unchanged)

## isEqual
Will compare one boolean to another.

```json
{
  "runtime_name": "std::number::isEqual",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The first operand in the equality check."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "NUMBER"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Number"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second number to compare."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The second operand in the equality check."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "error_type_identifiers": [],
  "name": [
    {
      "code": "en-US",
      "content": "Is Equal"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks whether two numbers are equal."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the first number is equal to the second number, otherwise false."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": []
}
```

**Example**:

1, 1 --> true

1, 2 --> false

# Additions:
- isGreaterOrEqual
- isLessOrEqual
