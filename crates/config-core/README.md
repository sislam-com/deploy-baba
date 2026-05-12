# config-core

Universal configuration parsing traits for zero-cost abstraction over multiple config formats.

## Usage

```rust
use config_core::{ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    name: String,
    port: u16,
}

struct MyParser;

impl ConfigParser<AppConfig> for MyParser {
    type Error = config_core::ConfigError;

    fn parse(input: &str) -> Result<AppConfig, Self::Error> {
        // Parse logic here
        todo!()
    }

    fn validate(config: &AppConfig) -> Result<(), Vec<ValidationError>> {
        if config.port == 0 {
            return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
        }
        Ok(())
    }
}
```

## Features

- `ConfigParser<T>` - Universal trait for parsing strings into typed configuration objects
- `ConfigValidator<T>` - Universal trait for validating configuration objects
- `ConfigMerger<T>` - Universal trait for merging multiple configurations
- `EnvironmentInterpolator<T>` - Universal trait for environment variable substitution
- `ConfigSource` - Enumeration of where configuration comes from
- `ValidationError` - Structured validation error with field and message

## License

MIT
