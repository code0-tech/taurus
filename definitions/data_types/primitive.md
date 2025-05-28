# All Data_Types of the Primitive Variant

## NUMBER

```json
{
  "variant": 1,
  "identifier": "NUMBER",
  "name": [
    {
      "code": "en-US",
      "content": "Number"
    }
  ],
  "rules": [
    {
      "regex": {
        "pattern": "/^(?:-(?:[1-9](?:\d{0,2}(?:,\d{3})+|\d*))|(?:0|(?:[1-9](?:\d{0,2}(?:,\d{3})+|\d*))))(?:.\d+|)$/"
      }
    }
  ],
  "generic_keys": []
}
```


## TEXT

```json
{
  "variant": 1,
  "identifier": "TEXT",
  "name": [
    {
      "code": "en-US",
      "content": "Text"
    }
  ],
  "rules": [
    {
      "regex": {
        "pattern": "[\s\S]*"
      }
    }
  ],
  "generic_keys": []
}
```

## BOOLEAN

```json
{
  "variant": 1,
  "identifier": "BOOLEAN",
  "name": [
    {
      "code": "en-US",
      "content": "Boolean"
    }
  ],
  "rules": [
    {
      "regex": {
        "pattern": "^(true|false)$"
      }
    }
  ],
  "generic_keys": []
}
```
