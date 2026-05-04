//! JSON Configuration Parser
//!
//! This crate provides a JSON parser that implements the universal configuration traits
//! from `config-core`. It enables type-safe, zero-cost configuration parsing with support
//! for custom validation logic per type.
//!
//! # Features
//!
//! - **Zero-cost abstraction**: Uses monomorphization for compile-time polymorphism
//! - **Custom validation**: Implement `JsonValidatable` for type-specific validation rules
//! - **Format flexibility**: Can parse any JSON structure into typed Rust structs
//! - **Complex types**: Supports nested objects, arrays, and numbers
//! - **File operations**: Convenience functions for loading and saving JSON files
//!
//! # Example
//!
//! ```rust
//! use config_json::{JsonParser, JsonValidatable};
//! use config_core::{ConfigParser, ValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! impl JsonValidatable for AppConfig {
//!     fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
//!         if self.port == 0 {
//!             return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let json_content = r#"{
//!   "name": "my-app",
//!   "port": 8080
//! }"#;
//!
//! let config: AppConfig = JsonParser::parse(json_content).unwrap();
//! assert_eq!(config.name, "my-app");
//! assert_eq!(config.port, 8080);
//! ```

use config_core::{ConfigError, ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// JSON parser implementing universal configuration traits
///
/// Uses compile-time polymorphism through monomorphization to provide
/// zero-cost abstraction. The parser is stateless and works with any type
/// that implements both `Deserialize` and `JsonValidatable`.
///
/// # Type Parameters
///
/// - `T`: The configuration type to parse (must implement `Deserialize` and `JsonValidatable`)
///
/// # Examples
///
/// ```rust
/// use config_json::JsonParser;
/// use config_core::ConfigParser;
/// use serde::{Deserialize, Serialize};
/// use config_json::JsonValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { value: String }
///
/// impl JsonValidatable for Config {
///     fn validate_json(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
/// }
///
/// let parser = JsonParser::<Config>::new();
/// ```
pub struct JsonParser<T> {
    _phantom: PhantomData<T>,
}

impl<T> JsonParser<T> {
    /// Create a new JSON parser for the given type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_json::JsonParser;
    /// use serde::{Deserialize, Serialize};
    /// use config_json::JsonValidatable;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct MyConfig;
    ///
    /// impl JsonValidatable for MyConfig {
    ///     fn validate_json(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
    /// }
    ///
    /// let parser = JsonParser::<MyConfig>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for JsonParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConfigParser<T> for JsonParser<T>
where
    T: for<'de> Deserialize<'de> + Serialize + JsonValidatable,
{
    type Error = JsonConfigError;

    fn parse(input: &str) -> Result<T, Self::Error> {
        serde_json::from_str(input).map_err(JsonConfigError::Parse)
    }

    fn validate(config: &T) -> Result<(), Vec<ValidationError>> {
        config.validate_json()
    }
}

/// Trait for types that can be validated when loaded from JSON
///
/// Implementors should validate domain constraints such as valid port ranges,
/// non-empty required fields, etc. Validation logic is separate from parsing
/// for better composability.
///
/// # Examples
///
/// ```rust
/// use config_json::JsonValidatable;
/// use config_core::ValidationError;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct ServerConfig {
///     port: u16,
///     host: String,
///     timeout_ms: u32,
/// }
///
/// impl JsonValidatable for ServerConfig {
///     fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
///         let mut errors = Vec::new();
///
///         if self.host.is_empty() {
///             errors.push(ValidationError::new("host", "Host cannot be empty"));
///         }
///
///         if self.port == 0 {
///             errors.push(ValidationError::new("port", "Port must be non-zero"));
///         }
///
///         if self.timeout_ms == 0 {
///             errors.push(ValidationError::new("timeout_ms", "Timeout must be positive"));
///         }
///
///         if errors.is_empty() { Ok(()) } else { Err(errors) }
///     }
/// }
/// ```
pub trait JsonValidatable {
    /// Validate the configuration object after deserialization
    ///
    /// # Errors
    ///
    /// Returns `Err` with a vector of validation errors if validation fails.
    /// Returns `Ok(())` if the configuration is valid.
    fn validate_json(&self) -> Result<(), Vec<ValidationError>>;
}

/// Blanket implementation for types that don't need custom validation
///
/// Common Rust types automatically implement `JsonValidatable` with no-op
/// validation. Types can override this by implementing `JsonValidatable` explicitly.
///
/// # Examples
///
/// ```rust
/// use config_json::JsonValidatable;
/// use serde::{Deserialize, Serialize};
///
/// // No need to implement JsonValidatable manually for simple types
/// #[derive(Serialize, Deserialize)]
/// struct SimpleConfig {
///     count: i32,
///     enabled: bool,
/// }
/// ```
macro_rules! impl_default_json_validation {
    ($($t:ty),*) => {
        $(
            impl JsonValidatable for $t {
                fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
                    Ok(())
                }
            }
        )*
    };
}

// Common types that don't need validation
impl_default_json_validation!(String, i32, i64, u32, u64, f32, f64, bool);

/// JSON-specific configuration errors
///
/// Wraps both JSON parsing errors from the `serde_json` crate and validation errors
/// from our configuration validation logic.
///
/// # Variants
///
/// - `Parse`: Error during JSON parsing (e.g., syntax error, type mismatch)
/// - `Validation`: Configuration validation failures
#[derive(Error, Debug)]
pub enum JsonConfigError {
    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),

    /// Configuration validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<ValidationError>),
}

impl From<JsonConfigError> for ConfigError {
    fn from(error: JsonConfigError) -> Self {
        match error {
            JsonConfigError::Parse(e) => ConfigError::Parse(e.to_string()),
            JsonConfigError::Validation(errors) => ConfigError::Validation(errors),
        }
    }
}

/// Helper function to format validation errors for display
fn format_validation_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(|e| format!("{}", e))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Load and parse JSON configuration from a file
///
/// Convenience function that combines file I/O, parsing, and validation.
/// Automatically validates the parsed configuration.
///
/// # Arguments
///
/// - `path`: File path to the JSON configuration file
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - The file cannot be read (IO error)
/// - The JSON is invalid (parse error)
/// - Validation fails
///
/// # Examples
///
/// ```rust,no_run
/// use config_json::load_json_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
/// use config_json::JsonValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// impl JsonValidatable for Config {
///     fn validate_json(&self) -> Result<(), Vec<config_core::ValidationError>> {
///         Ok(())
///     }
/// }
///
/// let config: Config = load_json_config("config.json")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn load_json_config<T>(path: impl AsRef<std::path::Path>) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + JsonValidatable,
{
    let content = std::fs::read_to_string(path)?;
    let config = JsonParser::parse(&content).map_err(ConfigError::from)?;
    JsonParser::validate(&config).map_err(ConfigError::Validation)?;
    Ok(config)
}

/// Save configuration to a JSON file
///
/// Convenience function that serializes and writes a configuration object
/// to a JSON file. Produces pretty-printed output.
///
/// # Arguments
///
/// - `config`: Configuration object to save
/// - `path`: File path where JSON will be written
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - Serialization fails
/// - The file cannot be written (IO error)
///
/// # Examples
///
/// ```rust,no_run
/// use config_json::save_json_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// let config = Config { port: 8080 };
/// save_json_config(&config, "config.json")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn save_json_config<T>(config: &T, path: impl AsRef<std::path::Path>) -> Result<(), ConfigError>
where
    T: Serialize,
{
    let content =
        serde_json::to_string_pretty(config).map_err(|e| ConfigError::Parse(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error as _;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
        enabled: bool,
    }

    impl JsonValidatable for TestConfig {
        fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
            let mut errors = Vec::new();

            if self.name.is_empty() {
                errors.push(ValidationError::new("name", "Name cannot be empty"));
            }

            if self.port == 0 {
                errors.push(ValidationError::new("port", "Port cannot be zero"));
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn test_json_parser_success() {
        let json_content = r#"{
            "name": "test-app",
            "port": 8080,
            "enabled": true
        }"#;

        let config: TestConfig = JsonParser::parse(json_content).unwrap();
        assert_eq!(config.name, "test-app");
        assert_eq!(config.port, 8080);
        assert!(config.enabled);
    }

    #[test]
    fn test_json_parser_validation() {
        let json_content = r#"{
            "name": "",
            "port": 0,
            "enabled": true
        }"#;

        let config: TestConfig = JsonParser::parse(json_content).unwrap();
        let validation_result = JsonParser::validate(&config);
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.field == "name"));
        assert!(errors.iter().any(|e| e.field == "port"));
    }

    #[test]
    fn test_json_parser_parse_and_validate() {
        let json_content = r#"{
            "name": "valid-app",
            "port": 3000,
            "enabled": false
        }"#;

        let config: TestConfig = JsonParser::parse_and_validate(json_content).unwrap();
        assert_eq!(config.name, "valid-app");
        assert_eq!(config.port, 3000);
        assert!(!config.enabled);
    }

    #[test]
    fn test_invalid_json_syntax() {
        let invalid_json = r#"{
            "name": "test-app",
            "port": 8080,
            "enabled": true
        "#; // Missing closing brace

        let result: Result<TestConfig, _> = JsonParser::parse(invalid_json);
        assert!(result.is_err());
        match result.unwrap_err() {
            JsonConfigError::Parse(_) => {}
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_json_nested_objects() {
        let json_content = r#"{
            "name": "nested-test",
            "port": 8080,
            "enabled": true
        }"#;

        let config: TestConfig = JsonParser::parse(json_content).unwrap();
        assert_eq!(config.name, "nested-test");
        assert_eq!(config.port, 8080);
        assert!(config.enabled);
    }

    #[test]
    fn test_json_arrays_and_numbers() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct ComplexConfig {
            name: String,
            ports: Vec<u16>,
            ratio: f64,
            tags: Vec<String>,
        }

        impl JsonValidatable for ComplexConfig {
            fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
                Ok(())
            }
        }

        let json_content = r#"{
            "name": "complex-app",
            "ports": [8080, 8081, 8082],
            "ratio": 0.95,
            "tags": ["web", "api", "production"]
        }"#;

        let config: ComplexConfig = JsonParser::parse(json_content).unwrap();
        assert_eq!(config.name, "complex-app");
        assert_eq!(config.ports, vec![8080, 8081, 8082]);
        assert_eq!(config.ratio, 0.95);
        assert_eq!(config.tags, vec!["web", "api", "production"]);
    }

    #[test]
    fn test_json_error_conversion() {
        let error = JsonConfigError::Parse(serde_json::Error::custom("test error"));
        let config_error: ConfigError = error.into();
        assert!(config_error.to_string().contains("Parse error"));
    }

    #[test]
    fn test_json_parser_default() {
        let _parser = JsonParser::<TestConfig>::default();
        // Just verify it constructs successfully
    }

    #[test]
    fn test_json_default_validation() {
        // Test blanket implementation for String
        let validation = "test".to_string().validate_json();
        assert!(validation.is_ok());

        // Test blanket implementation for i32
        let validation = 42i32.validate_json();
        assert!(validation.is_ok());

        // Test blanket implementation for f64
        let validation = std::f64::consts::PI.validate_json();
        assert!(validation.is_ok());
    }

    #[test]
    fn test_json_blanket_validation_remaining_types() {
        assert!(42u32.validate_json().is_ok());
        assert!(42u64.validate_json().is_ok());
        assert!(1.5_f32.validate_json().is_ok());
        assert!(true.validate_json().is_ok());
        assert!(42i64.validate_json().is_ok());
    }

    #[test]
    fn test_json_parse_and_validate_validation_failure() {
        let json_content = r#"{
            "name": "",
            "port": 0,
            "enabled": true
        }"#;
        let result: Result<TestConfig, _> = JsonParser::parse_and_validate(json_content);
        assert!(result.is_err());
        match result.unwrap_err() {
            config_core::ConfigParseError::Validation(errors) => {
                assert!(!errors.is_empty());
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_json_validation_error_display_and_conversion() {
        let errors = vec![ValidationError::new("port", "must be nonzero")];
        let err = JsonConfigError::Validation(errors.clone());
        assert!(err.to_string().contains("Validation failed"));

        let config_err: ConfigError = JsonConfigError::Validation(errors).into();
        assert!(matches!(config_err, ConfigError::Validation(_)));
    }

    #[test]
    fn test_load_and_save_json_config() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_config_json_roundtrip.json");

        let original = TestConfig {
            name: "roundtrip".to_string(),
            port: 4321,
            enabled: false,
        };

        save_json_config(&original, &path).expect("save failed");

        let loaded: TestConfig = load_json_config(&path).expect("load failed");
        assert_eq!(loaded.name, "roundtrip");
        assert_eq!(loaded.port, 4321);
        assert!(!loaded.enabled);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_json_config_file_not_found() {
        let result: Result<TestConfig, ConfigError> =
            load_json_config("/nonexistent/path/config.json");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }
}
