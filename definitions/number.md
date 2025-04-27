# std for PRIMITIVE: NUMBER

## add
Adds two numbers together.

```json
{
  "runtime_name": "std::number::add",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "base"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "exponent"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
  "return_type_identifier": "PRIMITIVE"
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
  "return_type_identifier": "PRIMITIVE"
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
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "decimals"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "decimals"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "decimals"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "root_exponent"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "base"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

2.71828... --> 1 (ln of e)

## parseNumber
Converts a string to a number.

```json
{
  "runtime_name": "std::number::parse_number",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "text"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

"123" --> 123
"3.14" --> 3.14

## min
Returns the smaller of two numbers.

```json
{
  "runtime_name": "std::number::min",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "min"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "max"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

1, 6 --> 3 (random number between 1 and 6, inclusive)

## sin
Calculates the sine of an angle (in radians).

```json
{
  "runtime_name": "std::number::sin",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "radians"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "radians"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "radians"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
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
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "value"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "min"
    },
    {
      "data_type_identifier": "PRIMITIVE",
      "runtime_name": "max"
    }
  ],
  "return_type_identifier": "PRIMITIVE"
}
```

**Example**:

10, 0, 5 --> 5 (clamped to max)
-3, 0, 5 --> 0 (clamped to min)
3, 0, 5 --> 3 (within range, unchanged)

# Additions:
- isGreaterOrEqual
- isLessOrEqual
