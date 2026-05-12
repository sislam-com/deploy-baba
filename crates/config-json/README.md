# config-json

JSON configuration parser implementing universal config traits for zero-cost abstraction.

## Usage

```rust
use config_json::{JsonParser, JsonValidatable};
use config_core::{ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    name: String,
    port: u16,
}

impl JsonValidatable for AppConfig {
    fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
        if self.port == 0 {
            return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
        }
        Ok(())
    }
}

let json_content = r#"{
  "name": "my-app",
  "port": 8080
}"#;

let config: AppConfig = JsonParser::parse(json_content).unwrap();
assert_eq!(config.name, "my-app");
assert_eq!(config.port, 8080);
```

## Features

- `JsonParser` - JSON parser implementing `ConfigParser` trait
- `JsonValidatable` - Trait for custom JSON validation logic
- Zero-cost abstraction via monomorphization
- Supports nested objects, arrays, and complex types
- File operations for loading and saving JSON files

## License

MIT
