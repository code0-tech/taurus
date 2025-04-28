# All Data_Types of the Primitive Variant

## NUMBER

```json
{
  "variant": "PRIMITIVE",
  "identifier": "NUMBER",
  "name": [
    {
      "code": "en-US",
      "content": "Number"
    }
  ],
  "rules": [
    {
      "config": {
        "regex": {
          "pattern": "/^(?:-(?:[1-9](?:\d{0,2}(?:,\d{3})+|\d*))|(?:0|(?:[1-9](?:\d{0,2}(?:,\d{3})+|\d*))))(?:.\d+|)$/"
        }
      }
    }
  ]
}
```


## TEXT

```json
{
  "variant": "PRIMITIVE",
  "identifier": "TEXT",
  "name": [
    {
      "code": "en-US",
      "content": "Text"
    }
  ],
  "rules": [
    {
      "config": {
        "regex": {
          "pattern": "[\s\S]*"
        }
      }
    }
  ]
}
```

## BOOLEAN

```json
{
  "variant": "PRIMITIVE",
  "identifier": "BOOLEAN",
  "name": [
    {
      "code": "en-US",
      "content": "Boolean"
    }
  ],
  "rules": [
    {
      "config": {
        "regex": {
          "pattern": "^(true|false)$"
        }
      }
    }
  ]
}
```
