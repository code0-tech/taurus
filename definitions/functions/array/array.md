## at
Will return the value at the index of the array.

```json
{
  "runtime_name": "std::array::at",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "NUMBER",
      "runtime_name": "index"
    }
  ],
  "return_type_identifier": "GENERIC"
}
```

## concat
Will merge to arrays together and return a new one

```json
{
  "runtime_name": "std::array::concat",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "first"
    },
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "second"
    }
  ],
  "return_type_identifier": "ARRAY"
}
```

## filter
Will filter the function by the given function

```json
{
  "runtime_name": "std::array::filter",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "FILTER_INPUT_GENERIC_NODE",
      "runtime_name": "fitler"
    }
  ],
  "return_type_identifier": "ARRAY"
}
```

## find
idk how to do functions

some node with a boolean as return type???

## findIndex
idk how to do functions

some node with a boolean as return type???

## first
idk how to do functions

some node with a boolean as return type???

## last
idk how to do functions

some node with a boolean as return type???

## forEach
idk how to do functions

some node with a boolean as return type???

## map
idk how to do functions

some node with a boolean as return type???

## push
Will add the given entry to the array and returns the new length of the array

```json
{
  "runtime_name": "std::array::filter",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    },
    {
      "data_type_identifier": "",
      "runtime_name": "value"
    }
  ],
  "return_type_identifier": "NUMBER"
}
```

[Ref](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push)

## pop
Will remove the last entry of the array and return it

```json
{
  "runtime_name": "std::array::pop",
  "runtime_parameter_definitions": [
    {
      "data_type_identifier": "ARRAY",
      "runtime_name": "array"
    }
  ],
  "return_type_identifier": "GENERIC"
}
```

[Ref](https://developer.mozilla.org/de/docs/Web/JavaScript/Reference/Global_Objects/Array/pop)

## remove

## isEmpty

## size

## indexOf

## toUnique

## sort

## sortASC

## sortDESC

## reverse

## flat

## clear

## replace

move to number_arry


min

max

sum

## join
will join every entry by a given text
