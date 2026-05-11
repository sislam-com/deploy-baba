# config-yaml

YAML configuration parser implementing universal config traits for zero-cost abstraction.

## Usage

```rust
use config_yaml::{YamlParser, YamlValidatable};
use config_core::{ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    name: String,
    port: u16,
}

impl YamlValidatable for AppConfig {
    fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
        if self.port == 0 {
            return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
        }
        Ok(())
    }
}

let yaml_content = r#"
name: my-app
port: 8080
"#;

let config: AppConfig = YamlParser::parse(yaml_content).unwrap();
assert_eq!(config.name, "my-app");
assert_eq!(config.port, 8080);
```

## Features

- `YamlParser` - YAML parser implementing `ConfigParser` trait
- `YamlValidatable` - Trait for custom YAML validation logic
- Zero-cost abstraction via monomorphization
- Full YAML features including multiline strings and arrays
- File operations for loading and saving YAML files

## License

MIT
