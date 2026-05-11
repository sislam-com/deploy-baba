# config-toml

TOML configuration parser implementing universal config traits for zero-cost abstraction.

## Usage

```rust
use config_toml::{TomlParser, TomlValidatable};
use config_core::{ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    name: String,
    port: u16,
}

impl TomlValidatable for AppConfig {
    fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if self.port == 0 {
            errors.push(ValidationError::new("port", "Port must be non-zero"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

let toml_content = r#"
name = "my-app"
port = 8080
"#;

let config: AppConfig = TomlParser::parse(toml_content).unwrap();
assert_eq!(config.name, "my-app");
assert_eq!(config.port, 8080);
```

## Features

- `TomlParser` - TOML parser implementing `ConfigParser` trait
- `TomlValidatable` - Trait for custom TOML validation logic
- Zero-cost abstraction via monomorphization
- File operations for loading and saving TOML files

## License

MIT
