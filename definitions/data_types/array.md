# All Data_Types of the Array Variant

## ARRAY
```json
{
  "variant": "ARRAY",
  "identifier": "ARRAY",
  "name": [
    {
      "code": "en-US",
      "content": "Array"
    }
  ],
}
```
## NUMBER_ARRAY

```json
{
  "variant": "ARRAY",
  "identifier": "NUMBER_ARRAY",
  "name": [
    {
      "code": "en-US",
      "content": "Number Array"
    }
  ],
  "rules": [
    {
      "config": {
        "contains_type": {
          "data_type_identifier": "NUMBER"
        }
      }
    }
  ],
  "parent_type_identifier": "ARRAY"
}
```

## TEXT_ARRAY

```json
{
  "variant": "ARRAY",
  "identifier": "TEXT_ARRAY",
  "name": [
    {
      "code": "en-US",
      "content": "Text Array"
    }
  ],
  "rules": [
    {
      "config": {
        "contains_type": {
          "data_type_identifier": "TEXT"
        }
      }
    }
  ],
  "parent_type_identifier": "ARRAY"
}
```

## BOOLEAN_ARRAY

```json
{
  "variant": "ARRAY",
  "identifier": "BOOLEAN_ARRAY",
  "name": [
    {
      "code": "en-US",
      "content": "Boolean Array"
    }
  ],
  "rules": [
    {
      "config": {
        "contains_type": {
          "data_type_identifier": "BOOLEAN"
        }
      }
    }
  ],
  "parent_type_identifier": "ARRAY"
}
```
