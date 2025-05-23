## at
Will return the value at the index of the array.

```json
{
  "runtime_name": "std::array::at",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
          "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "index"
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mapper": [
    {
      "parameter_id": "array",
      "soruce": "R",
      "target": "T"
    }
  ]
}
```

## concat
Will merge to arrays together and return a new one.

```json
{
  "runtime_name": "std::array::concat",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "first"
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "second"
    }
  ],
  "data_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "first",
      "soruce": "R",
      "target": "T"
    },
    {
      "parameter_id": "second",
      "soruce": "R",
      "target": "T"
    },
    {
      "soruce": "R",
      "target": "T"
    }
  ]
}
```

## filter
Will filter the array by the given node and return all values, the filter node returned true.

```json
{
  "runtime_name": "std::array::filter",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "PREDICATE",
      "runtime_name": "predicate"
    }
  ],
  "return_type_identifier": "ARRAY",
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": "R",
      "target": "T"
    },
    {
      "source": "R",
      "target": "T"
    }
  ]
}
```

## find
Will return the first item of an array that match the predicate.

```json
{
  "runtime_name": "std::array::find",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "PREDICATE",
      "runtime_name": "predicate"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": "R",
      "target": "T"
    },
  ]
}
```

## findLast
Will return the last item of an array that match the predicate.

```json
{
  "runtime_name": "std::array::find_last",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "PREDICATE",
      "runtime_name": "predicate"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": "R",
      "target": "T"
    },
  ]
}
```

## findIndex
Will return the index of the first item that matches the predicate.

```json
{
  "runtime_name": "std::array::find_index",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "PREDICATE",
      "runtime_name": "predicate"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": "R",
      "target": "T"
    },
  ]
}
```

## first
Will return the first item of the array.

```json
{
  "runtime_name": "std::array::first",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    {
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "soruce": "R",
      "target": "T"
    }
  ]
}
```

## last
Will return the last item of the array.

```json
{
  "runtime_name": "std::array::last",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "soruce": "R",
      "target": "T"
    }
  ]
}
```

## forEach
Will call a consumer on every item in the array. No return value.

```json
{
  "runtime_name": "std::array::for_each",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "CONSUMER",
      "runtime_name": "consumer"
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "consumer",
      "source": "R",
      "target": "T"
    },
  ]
}
```

## map
Will call a node on each value and expect a return value of the node, collect all return values and returns a new array of all collected return values.

```json
{
  "runtime_name": "std::array::map",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "TRANSFORM",
      "runtime_name": "transform"
    }
  ],
  "generic_keys": ["IN", "OUT"],
  "return_type_identifier": "ARRAY",
  "generic_mappers": [
    {
      "parameter_id": "array",
      "soruce": "IN",
      "target": "T"
    },
    {
      "parameter_id": "transform",
      "source": "IN", // <-- Same type as item of array
      "target": "I" // <-- input type of transform node
    },
    {
      "parameter_id": "transform",
      "soruce": "OUT", // <-- Return type of node
      "target": "R" // <-- Return type of transform node
    },
    {
      "soruce": "OUT",
      "target": "T"
    }
  ]
}
```

## push
Will add the given item to the array and returns the new length of the array.

```json
{
  "runtime_name": "std::array::push",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": {
        "generic_key": "I"
      },
      "runtime_name": "item"
    }
  ],
  "return_type_identifier": "NUMBER",
  "generic_keys": ["I"],
  "generic_mapper": [
    {
      "parameter_id": "array",
      "source": "T",
      "target": "I"
    }
  ]
}
```

[Ref](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push)

## pop
Will remove the last entry of the array and return the item.

```json
{
  "runtime_name": "std::array::pop",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_mapper": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

[Ref](https://developer.mozilla.org/de/docs/Web/JavaScript/Reference/Global_Objects/Array/pop)

## remove
Will remove the given item of the array and return the array without it.

```json
{
  "runtime_name": "std::array::remove",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": {
        "generic_key": "R"
      },
      "runtime_name": "item"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": "ARRAY",
  "generic_mapper": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

## isEmpty
Will check if the array is empty or not. Will return true if its empty.

```json
{
  "runtime_name": "std::array::is_empty",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "BOOLEAN",
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

## size
Will return the amount of items inside the array.

```json
{
  "runtime_name": "std::array::size",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "NUMBER",
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

## indexOf
Will return the index of the given item.

```json
{
  "runtime_name": "std::array::index_of",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": {
        "generic_key": "R"
      },
      "runtime_name": "item"
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": "NUMBER",
  "generic_mapper": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

## toUnique
Will remove all duplicated items of the array.

```json
{
  "runtime_name": "std::array::to_unique",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "soruce": "R",
      "target": "T"
    },
    {
      "soruce": "R",
      "target": "T"
    }
  ]
}
```

## sort
Will sort the array by the given COMPARATOR.

```json
{
  "runtime_name": "std::array::sort",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
    {
      "data_type_identifier": "COMPARATOR",
      "runtime_name": "COMPARATOR"
    }
  ],
  "return_type_identifier": "ARRAY",
  "generic_keys": "R",
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "COMPARATOR",
      "source": "R",
      "target": "I"
    }
  ]
}
```

## sortReverse

```json
{
  "runtime_name": "std::array::sort_reverse",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
    {
      "data_type_identifier": "COMPARATOR",
      "runtime_name": "comparator"
    }
  ],
  "return_type_identifier": "ARRAY",
  "generic_keys": "R",
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    },
    {
      "parameter_id": "comparator",
      "source": "R",
      "target": "I"
    }
  ]
}
```

## reverse

```json
{
  "runtime_name": "std::array::reverse",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "ARRAY",
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": "R",
      "target": "T"
    }
  ]
}
```

## flat
Will turn a 2 dimensional array into a one dimensional array.

Input:
[ [1, 2, 3], [3, 4, 5] ]

Result:
[ 1, 2, 3, 3, 4, 5 ]

```json
{
  "runtime_name": "std::array::flat",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "generic_type": {
          "data_type_identifier": "ARRAY",
          "generic_mapper": {
            "data_type_identifier": "ARRAY",
            "target": "T"
          }
        }
      },
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "ARRAY",
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "filter",
      "target": "R",
      "soruce": "T"
    }
  ]
}
```

## min
Returns the smallest number in the array

```json
{
  "runtime_name": "std::array::min",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "generic_type": {
          "data_type_identifier": "ARRAY",
          "generic_mapper": {
            "data_type_identifier": "NUMBER",
            "target": "T"
          }
        }
      },
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "NUMBER"
}
```

## max
Returns the largest number in the array

```json
{
  "runtime_name": "std::array::max",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "generic_type": {
          "data_type_identifier": "ARRAY",
          "generic_mapper": {
            "data_type_identifier": "NUMBER",
            "target": "T"
          }
        }
      },
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "NUMBER"
}
```


## sum
Returns the sum of all the numbers in the array

```json
{
  "runtime_name": "std::array::sum",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "generic_type": {
          "data_type_identifier": "ARRAY",
          "generic_mapper": {
            "data_type_identifier": "NUMBER",
            "target": "T"
          }
        }
      },
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "NUMBER"
}
```

## join
Will join every item by a given text

```json
{
  "runtime_name": "std::array::filter",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "generic_type": {
          "data_type_identifier": "ARRAY",
          "generic_mapper": {
            "data_type_identifier": "TEXT",
            "target": "T"
          }
        }
      },
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "TEXT",
      "runtime_name": "join_text"
    }
  ],
  "return_type_identifier": "TEXT"
}
```
