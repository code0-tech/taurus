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
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array from which to retrieve an element."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array containing elements of any type. The element at the specified index will be returned."
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
          "content": "The zero-based index of the element to retrieve."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies the position of the element in the array to return. Must be within the bounds of the array."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "name": [
    {
      "code": "en-US",
      "content": "Get Array Element"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Retrieves the element at a specified index from an array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the element located at the given zero-based index within the input array."
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
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
      "runtime_name": "first",
      "name": [
        {
          "code": "en-US",
          "content": "First Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The first array to concatenate."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The first input array whose elements will appear at the beginning of the resulting array."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "second",
      "name": [
        {
          "code": "en-US",
          "content": "Second Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The second array to concatenate."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The second input array whose elements will be appended after the elements of the first array."
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
      "content": "Concatenate Arrays"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Concatenates two arrays into a single array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array containing all elements of the first array followed by all elements of the second array."
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "first",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "second",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## filter
Will filter the array by the given node and return all values, the filter node returned true.

```json
{
  "runtime_name": "std::array::filter",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to be filtered."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The original array from which elements will be selected based on the predicate."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "PREDICATE"
      },
      "runtime_name": "predicate",
      "name": [
        {
          "code": "en-US",
          "content": "Filter Predicate"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A predicate function to test each element for inclusion in the result."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes an element of the array and returns a boolean indicating whether the element should be included in the output array."
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
      "content": "Filter Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Filters elements of an array based on a predicate."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array containing only the elements from the input array for which the predicate returns true."
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## find
Will return the first item of an array that match the predicate.

```json
{
  "runtime_name": "std::array::find",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to search through."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The array in which an element satisfying the predicate will be searched."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "PREDICATE"
      },
      "runtime_name": "predicate",
      "name": [
        {
          "code": "en-US",
          "content": "Search Predicate"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A predicate function used to test each element for a match."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes an element of the array and returns a boolean indicating if the element matches the search criteria."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Find Element in Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the first element in the array that satisfies the predicate."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the first element from the input array for which the predicate returns true. If no element matches, returns null or equivalent."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## findLast
Will return the last item of an array that match the predicate.

```json
{
  "runtime_name": "std::array::find_last",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to search through."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The array in which an element satisfying the predicate will be searched."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "PREDICATE"
      },
      "runtime_name": "predicate",
      "name": [
        {
          "code": "en-US",
          "content": "Search Predicate"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A predicate function used to test each element for a match."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes an element of the array and returns a boolean indicating if the element matches the search criteria."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Find Last Element in Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the last element in the array that satisfies the predicate."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the last element from the input array for which the predicate returns true. If no element matches, returns null or equivalent."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## findIndex
Will return the index of the first item that matches the predicate.

```json
{
  "runtime_name": "std::array::find_index",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to search through."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The array in which to find the index of an element that satisfies the predicate."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "PREDICATE"
      },
      "runtime_name": "predicate",
      "name": [
        {
          "code": "en-US",
          "content": "Search Predicate"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A predicate function used to test each element for a match."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes an element of the array and returns a boolean indicating if the element satisfies the search criteria."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "predicate",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Find Index in Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the index of the first element in the array that satisfies the predicate."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the zero-based index of the first element for which the predicate returns true. If no element matches, returns -1."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## first
Will return the first item of the array.

```json
{
  "runtime_name": "std::array::first",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array from which to retrieve the first element."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Returns the first element of the provided array. If the array is empty, behavior depends on the implementation."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "First Element of Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Retrieves the first element from the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "This runtime returns the first element in the given array, if any."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## last
Will return the last item of the array.

```json
{
  "runtime_name": "std::array::last",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array from which to retrieve the last element."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Returns the last element of the provided array. If the array is empty, behavior depends on the implementation."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Last Element of Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Retrieves the last element from the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "This runtime returns the last element in the given array, if any."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## forEach
Will call a consumer on every item in the array. No return value.

```json
{
  "runtime_name": "std::array::for_each",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array of elements to iterate over."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Each element of this array will be passed to the provided consumer function for processing."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "CONSUMER"
      },
      "runtime_name": "consumer",
      "name": [
        {
          "code": "en-US",
          "content": "Consumer Function"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A function that consumes each element of the array."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This function is invoked once for each element in the array. It is not expected to return a value."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "consumer",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "For Each Element"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Executes a consumer function for each element in the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "This runtime executes the given consumer function on each item in the array without returning a result."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## map
Will call a node on each value and expect a return value of the node, collect all return values and returns a new array of all collected return values.

```json
{
  "runtime_name": "std::array::map",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Input Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to be transformed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Each element of this array will be passed through the transform function."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TRANSFORM"
      },
      "runtime_name": "transform",
      "name": [
        {
          "code": "en-US",
          "content": "Transform Function"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A function that transforms each item in the array."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The transform function is applied to every element of the array to produce a new array."
        }
      ]
    }
  ],
  "generic_keys": ["IN", "OUT"],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "IN"
      },
      "target": "T"
    },
    {
      "parameter_id": "transform",
      "source": {
        "generic_key": "IN"
      },
      "target": "I"
    },
    {
      "parameter_id": "transform",
      "source": {
        "generic_key": "OUT"
      },
      "target": "R"
    },
    {
      "source": {
        "generic_key": "OUT"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Map Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Transforms each element in the array using the provided function."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "This runtime applies the transform function to each element in the array, producing a new array of the results."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## push
Will add the given item to the array and returns the new length of the array.

```json
{
  "runtime_name": "std::array::push",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to which an item will be added."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The array that the new item will be appended to."
        }
      ]
    },
    {
      "data_type_identifier": {
        "generic_key": "I"
      },
      "runtime_name": "item",
      "name": [
        {
          "code": "en-US",
          "content": "Item"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The item to add to the array."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The value to be added at the end of the array."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "generic_keys": ["I"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "T"
      },
      "target": "I"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Push to Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Appends an item to the end of an array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Adds a new element to the end of the array and returns the new length of the array."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
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
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to remove the last item from."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "This is the array from which the last element will be removed and returned."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "generic_key": "R"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Pop from Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Removes and returns the last item from the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Removes the last element from the specified array and returns it. The array is modified in place."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
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
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array from which the item will be removed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array to process by removing the first occurrence of the specified item."
        }
      ]
    },
    {
      "data_type_identifier": {
        "generic_key": "R"
      },
      "runtime_name": "item",
      "name": [
        {
          "code": "en-US",
          "content": "Item"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The item to remove from the array."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The value to search for and remove from the array. Only the first matching item is removed."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "data_type_identifier": "ARRAY"
  },
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Remove from Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Removes the first occurrence of the specified item from the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Removes the first matching item from the given array and returns the resulting array."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## isEmpty
Will check if the array is empty or not. Will return true if its empty.

```json
{
  "runtime_name": "std::array::is_empty",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to check for emptiness."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The array whose length will be evaluated to determine if it contains any elements."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "BOOLEAN"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Is Array Empty"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Checks if the array has no elements."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns true if the array contains no elements, otherwise returns false."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## size
Will return the amount of items inside the array.

```json
{
  "runtime_name": "std::array::size",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array whose number of elements is to be returned."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Specifies the array for which the total number of elements will be calculated and returned."
        }
      ]
    }
  ],
  "return_type_identifier": {
    "data_type_identifier": "NUMBER"
  },
  "generic_keys": ["R"],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Array Size"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the number of elements in the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "This function returns the count of elements present in the given array."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## indexOf
Will return the index of the given item.

```json
{
  "runtime_name": "std::array::index_of",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The array to search within."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array of elements in which the specified item will be searched for to determine its index."
        }
      ]
    },
    {
      "data_type_identifier": {
        "generic_key": "R"
      },
      "runtime_name": "item",
      "name": [
        {
          "code": "en-US",
          "content": "Item"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The item whose index is to be found in the array."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The item for which the function searches in the array and returns the index of its first occurrence."
        }
      ]
    }
  ],
  "generic_keys": ["R"],
  "return_type_identifier": {
    "data_type_identifier":  "NUMBER"
  },
  "generic_mapper": [
    {
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Index of Item"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the index of the first occurrence of the specified item in the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the zero-based index of the first occurrence of a given item in the specified array. If the item is not found, it typically returns -1."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_mappers": []
}
```

## toUnique
Will remove all duplicated items of the array.

```json
{
  "runtime_name": "std::array::to_unique",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input array from which duplicates will be removed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array of elements that may contain duplicates. This function will remove any duplicate items and return a new array with unique values only."
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
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "To Unique"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Removes duplicate elements from the input array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array containing only the unique elements from the input array. The original order may or may not be preserved depending on the implementation."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## sort
Will sort the array by the given COMPARATOR.

```json
{
  "runtime_name": "std::array::sort",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input array to be sorted."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array of elements that will be sorted using the provided comparator function."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "COMPARATOR"
      },
      "runtime_name": "comparator",
      "name": [
        {
          "code": "en-US",
          "content": "Comparator"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A comparator function used to determine the sort order of elements."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes two elements and returns a negative, zero, or positive number to indicate their ordering."
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
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "comparator",
      "source": {
        "generic_key": "R"
      },
      "target": "I"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Sort Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Sorts the elements of the array using the specified comparator."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array with the elements sorted according to the comparator function provided."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## sortReverse

```json
{
  "runtime_name": "std::array::sort_reverse",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input array to be sorted in reverse order."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array of elements that will be sorted in descending order using the provided comparator."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "COMPARATOR"
      },
      "runtime_name": "comparator",
      "name": [
        {
          "code": "en-US",
          "content": "Comparator"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "A comparator function used to determine the sort order of elements."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "A function that takes two elements and returns a negative, zero, or positive number to indicate their ordering."
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
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    },
    {
      "parameter_id": "comparator",
      "source": {
        "generic_key": "R"
      },
      "target": "I"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Sort Array in Reverse"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Sorts the elements of the array in reverse order using the specified comparator."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array with the elements sorted in descending order according to the comparator function provided."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
}
```

## reverse

```json
{
  "runtime_name": "std::array::reverse",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The input array to be reversed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array of elements whose order will be reversed."
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
      "parameter_id": "array",
      "source": {
        "generic_key": "R"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Reverse Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Reverses the order of elements in the array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array with the elements of the input array in reverse order."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
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
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Nested Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "The nested array to be flattened."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "An array containing sub-arrays that will be flattened into a single-level array."
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
      "parameter_id": "filter",
      "target": "R",
      "soruce": "T"
    },
    {
      "parameter_id": "array",
      "source": {
        "data_type_identifier": "ARRAY"
      },
      "target": "T"
    }
  ],
  "name": [
    {
      "code": "en-US",
      "content": "Flatten Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Flattens a nested array into a single-level array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns a new array by concatenating all sub-arrays of the input nested array into one flat array."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": []
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
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Number Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "An array of numbers to find the minimum value from."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Returns the smallest number in the given array of numbers."
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
      "content": "Find Minimum Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the minimum value in a numeric array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the smallest number contained in the provided array."
    }
  ],
  "deprecation_message": [],
  "error_type_identifiers": [],
  "generic_keys": [],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "T"
    }
  ]
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
        "data_type_identifier": "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Number Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "An array of numbers to find the maximum value from."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Returns the largest number in the given array of numbers."
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
      "content": "Find Maximum Number"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Finds the maximum value in a numeric array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Returns the largest number contained in the provided array."
    }
  ],
  "deprecation_message": [],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "T"
    }
  ],
  "generic_keys": [],
  "error_type_identifiers": []
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
        "data_type_identifier":  "ARRAY"
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Number Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "An array of numbers to be summed."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Calculates the sum of all numbers in the given array."
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
      "content": "Sum of Numbers"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Returns the total sum of the elements in the numeric array."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Adds up all numbers in the input array and returns their sum."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "error_type_identifiers": [],
  "generic_mappers": [
    {
      "parameter_id": "array",
      "source": {
        "data_type_identifier": "NUMBER"
      },
      "target": "T"
    }
  ]
}
```

## join
Will join every item by a given text

```json
{
  "runtime_name": "std::array::join",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": {
        "data_type_identifier": {
          "data_type_identifier": "ARRAY"
        },
        "generic_mappers": [
          {
            "source": {
              "data_type_identifier": "TEXT"
            },
            "target": "T"
          }
        ]
      },
      "runtime_name": "array",
      "name": [
        {
          "code": "en-US",
          "content": "Text Array"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "An array of text elements to be filtered."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "Input array containing text elements for filtering."
        }
      ]
    },
    {
      "data_type_identifier": {
        "data_type_identifier": "TEXT"
      },
      "runtime_name": "join_text",
      "name": [
        {
          "code": "en-US",
          "content": "Join Text"
        }
      ],
      "description": [
        {
          "code": "en-US",
          "content": "Text to join the filtered elements."
        }
      ],
      "documentation": [
        {
          "code": "en-US",
          "content": "The delimiter or text that will be used to join the filtered array elements into a single string."
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
      "content": "Filter and Join Text Array"
    }
  ],
  "description": [
    {
      "code": "en-US",
      "content": "Filters the input text array and joins the filtered elements into a single string separated by the specified join text."
    }
  ],
  "documentation": [
    {
      "code": "en-US",
      "content": "Applies a filter operation on the input text array and returns a single concatenated string of filtered elements joined by the provided join text."
    }
  ],
  "deprecation_message": [],
  "generic_keys": [],
  "generic_mappers": [],
  "error_type_identifiers": []
}
```
