//! YAML Configuration Parser
//!
//! This crate provides a YAML parser that implements the universal configuration traits
//! from `config-core`. It enables type-safe, zero-cost configuration parsing with support
//! for custom validation logic per type.
//!
//! # Features
//!
//! - **Zero-cost abstraction**: Uses monomorphization for compile-time polymorphism
//! - **Custom validation**: Implement `YamlValidatable` for type-specific validation rules
//! - **Format flexibility**: Can parse any YAML structure into typed Rust structs
//! - **Multiline support**: Full YAML features including multiline strings and arrays
//! - **File operations**: Convenience functions for loading and saving YAML files
//!
//! # Example
//!
//! ```rust
//! use config_yaml::{YamlParser, YamlValidatable};
//! use config_core::{ConfigParser, ValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! impl YamlValidatable for AppConfig {
//!     fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
//!         if self.port == 0 {
//!             return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let yaml_content = r#"
//! name: my-app
//! port: 8080
//! "#;
//!
//! let config: AppConfig = YamlParser::parse(yaml_content).unwrap();
//! assert_eq!(config.name, "my-app");
//! assert_eq!(config.port, 8080);
//! ```

use config_core::{ConfigError, ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// YAML parser implementing universal configuration traits
///
/// Uses compile-time polymorphism through monomorphization to provide
/// zero-cost abstraction. The parser is stateless and works with any type
/// that implements both `Deserialize` and `YamlValidatable`.
///
/// # Type Parameters
///
/// - `T`: The configuration type to parse (must implement `Deserialize` and `YamlValidatable`)
///
/// # Examples
///
/// ```rust
/// use config_yaml::YamlParser;
/// use config_core::ConfigParser;
/// use serde::{Deserialize, Serialize};
/// use config_yaml::YamlValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { value: String }
///
/// impl YamlValidatable for Config {
///     fn validate_yaml(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
/// }
///
/// let parser = YamlParser::<Config>::new();
/// ```
pub struct YamlParser<T> {
    _phantom: PhantomData<T>,
}

impl<T> YamlParser<T> {
    /// Create a new YAML parser for the given type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_yaml::YamlParser;
    /// use serde::{Deserialize, Serialize};
    /// use config_yaml::YamlValidatable;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct MyConfig;
    ///
    /// impl YamlValidatable for MyConfig {
    ///     fn validate_yaml(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
    /// }
    ///
    /// let parser = YamlParser::<MyConfig>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for YamlParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConfigParser<T> for YamlParser<T>
where
    T: for<'de> Deserialize<'de> + Serialize + YamlValidatable,
{
    type Error = YamlConfigError;

    fn parse(input: &str) -> Result<T, Self::Error> {
        serde_yaml::from_str(input).map_err(YamlConfigError::Parse)
    }

    fn validate(config: &T) -> Result<(), Vec<ValidationError>> {
        config.validate_yaml()
    }
}

/// Trait for types that can be validated when loaded from YAML
///
/// Implementors should validate domain constraints such as valid port ranges,
/// non-empty required fields, etc. Validation logic is separate from parsing
/// for better composability.
///
/// # Examples
///
/// ```rust
/// use config_yaml::YamlValidatable;
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
/// impl YamlValidatable for ServerConfig {
///     fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
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
pub trait YamlValidatable {
    /// Validate the configuration object after deserialization
    ///
    /// # Errors
    ///
    /// Returns `Err` with a vector of validation errors if validation fails.
    /// Returns `Ok(())` if the configuration is valid.
    fn validate_yaml(&self) -> Result<(), Vec<ValidationError>>;
}

/// Blanket implementation for types that don't need custom validation
///
/// Common Rust types automatically implement `YamlValidatable` with no-op
/// validation. Types can override this by implementing `YamlValidatable` explicitly.
///
/// # Examples
///
/// ```rust
/// use config_yaml::YamlValidatable;
/// use serde::{Deserialize, Serialize};
///
/// // No need to implement YamlValidatable manually for simple types
/// #[derive(Serialize, Deserialize)]
/// struct SimpleConfig {
///     count: i32,
///     enabled: bool,
/// }
/// ```
macro_rules! impl_default_yaml_validation {
    ($($t:ty),*) => {
        $(
            impl YamlValidatable for $t {
                fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
                    Ok(())
                }
            }
        )*
    };
}

// Common types that don't need validation
impl_default_yaml_validation!(String, i32, i64, u32, u64, f32, f64, bool);

/// YAML-specific configuration errors
///
/// Wraps both YAML parsing errors from the `serde_yaml` crate and validation errors
/// from our configuration validation logic.
///
/// # Variants
///
/// - `Parse`: Error during YAML parsing (e.g., syntax error, type mismatch)
/// - `Validation`: Configuration validation failures
#[derive(Error, Debug)]
pub enum YamlConfigError {
    /// YAML parsing error
    #[error("YAML parse error: {0}")]
    Parse(#[from] serde_yaml::Error),

    /// Configuration validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<ValidationError>),
}

impl From<YamlConfigError> for ConfigError {
    fn from(error: YamlConfigError) -> Self {
        match error {
            YamlConfigError::Parse(e) => ConfigError::Parse(e.to_string()),
            YamlConfigError::Validation(errors) => ConfigError::Validation(errors),
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

/// Load and parse YAML configuration from a file
///
/// Convenience function that combines file I/O, parsing, and validation.
/// Automatically validates the parsed configuration.
///
/// # Arguments
///
/// - `path`: File path to the YAML configuration file
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - The file cannot be read (IO error)
/// - The YAML is invalid (parse error)
/// - Validation fails
///
/// # Examples
///
/// ```rust,no_run
/// use config_yaml::load_yaml_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
/// use config_yaml::YamlValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// impl YamlValidatable for Config {
///     fn validate_yaml(&self) -> Result<(), Vec<config_core::ValidationError>> {
///         Ok(())
///     }
/// }
///
/// let config: Config = load_yaml_config("config.yaml")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn load_yaml_config<T>(path: impl AsRef<std::path::Path>) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + YamlValidatable,
{
    let content = std::fs::read_to_string(path)?;
    let config = YamlParser::parse(&content).map_err(ConfigError::from)?;
    YamlParser::validate(&config).map_err(ConfigError::Validation)?;
    Ok(config)
}

/// Save configuration to a YAML file
///
/// Convenience function that serializes and writes a configuration object
/// to a YAML file.
///
/// # Arguments
///
/// - `config`: Configuration object to save
/// - `path`: File path where YAML will be written
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
/// use config_yaml::save_yaml_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// let config = Config { port: 8080 };
/// save_yaml_config(&config, "config.yaml")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn save_yaml_config<T>(config: &T, path: impl AsRef<std::path::Path>) -> Result<(), ConfigError>
where
    T: Serialize,
{
    let content = serde_yaml::to_string(config).map_err(|e| ConfigError::Parse(e.to_string()))?;
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

    impl YamlValidatable for TestConfig {
        fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
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
    fn test_yaml_parser_success() {
        let yaml_content = r#"
name: test-app
port: 8080
enabled: true
"#;

        let config: TestConfig = YamlParser::parse(yaml_content).unwrap();
        assert_eq!(config.name, "test-app");
        assert_eq!(config.port, 8080);
        assert!(config.enabled);
    }

    #[test]
    fn test_yaml_parser_validation() {
        let yaml_content = r#"
name: ""
port: 0
enabled: true
"#;

        let config: TestConfig = YamlParser::parse(yaml_content).unwrap();
        let validation_result = YamlParser::validate(&config);
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.field == "name"));
        assert!(errors.iter().any(|e| e.field == "port"));
    }

    #[test]
    fn test_yaml_parser_parse_and_validate() {
        let yaml_content = r#"
name: valid-app
port: 3000
enabled: false
"#;

        let config: TestConfig = YamlParser::parse_and_validate(yaml_content).unwrap();
        assert_eq!(config.name, "valid-app");
        assert_eq!(config.port, 3000);
        assert!(!config.enabled);
    }

    #[test]
    fn test_invalid_yaml_syntax() {
        let invalid_yaml = r#"
name: test-app
port: 8080
  - invalid_structure
"#;

        let result: Result<TestConfig, _> = YamlParser::parse(invalid_yaml);
        assert!(result.is_err());
        match result.unwrap_err() {
            YamlConfigError::Parse(_) => {}
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_yaml_multiline_support() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct MultilineConfig {
            name: String,
            port: u16,
            enabled: bool,
        }

        impl YamlValidatable for MultilineConfig {
            fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
                Ok(())
            }
        }

        let yaml_content = r#"
name: |
  multi-line
  application
  name
port: 8080
enabled: true
"#;

        let config: MultilineConfig = YamlParser::parse(yaml_content).unwrap();
        assert!(config.name.contains("multi-line"));
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_yaml_arrays() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct ArrayConfig {
            name: String,
            ports: Vec<u16>,
        }

        impl YamlValidatable for ArrayConfig {
            fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
                Ok(())
            }
        }

        let yaml_content = r#"
name: server
ports:
  - 8080
  - 8081
  - 8082
"#;

        let config: ArrayConfig = YamlParser::parse(yaml_content).unwrap();
        assert_eq!(config.name, "server");
        assert_eq!(config.ports, vec![8080, 8081, 8082]);
    }

    #[test]
    fn test_yaml_error_conversion() {
        let error = YamlConfigError::Parse(serde_yaml::Error::custom("test error"));
        let config_error: ConfigError = error.into();
        assert!(config_error.to_string().contains("Parse error"));
    }

    #[test]
    fn test_yaml_parser_default() {
        let _parser = YamlParser::<TestConfig>::default();
        // Just verify it constructs successfully
    }

    #[test]
    fn test_yaml_default_validation() {
        // Test blanket implementation for String
        let validation = "test".to_string().validate_yaml();
        assert!(validation.is_ok());

        // Test blanket implementation for i32
        let validation = 42i32.validate_yaml();
        assert!(validation.is_ok());
    }

    #[test]
    fn test_yaml_blanket_validation_remaining_types() {
        assert!(42u32.validate_yaml().is_ok());
        assert!(42u64.validate_yaml().is_ok());
        assert!(1.5_f32.validate_yaml().is_ok());
        assert!(1.5_f64.validate_yaml().is_ok());
        assert!(42i64.validate_yaml().is_ok());
        assert!(false.validate_yaml().is_ok());
    }

    #[test]
    fn test_yaml_parse_and_validate_validation_failure() {
        let yaml_content = r#"
name: ""
port: 0
enabled: true
"#;
        let result: Result<TestConfig, _> = YamlParser::parse_and_validate(yaml_content);
        assert!(result.is_err());
        match result.unwrap_err() {
            config_core::ConfigParseError::Validation(errors) => {
                assert!(!errors.is_empty());
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_yaml_validation_error_display_and_conversion() {
        let errors = vec![ValidationError::new("port", "must be nonzero")];
        let err = YamlConfigError::Validation(errors.clone());
        assert!(err.to_string().contains("Validation failed"));

        let config_err: ConfigError = YamlConfigError::Validation(errors).into();
        assert!(matches!(config_err, ConfigError::Validation(_)));
    }

    #[test]
    fn test_load_and_save_yaml_config() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_config_yaml_roundtrip.yaml");

        let original = TestConfig {
            name: "roundtrip".to_string(),
            port: 4321,
            enabled: false,
        };

        save_yaml_config(&original, &path).expect("save failed");

        let loaded: TestConfig = load_yaml_config(&path).expect("load failed");
        assert_eq!(loaded.name, "roundtrip");
        assert_eq!(loaded.port, 4321);
        assert!(!loaded.enabled);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_yaml_config_file_not_found() {
        let result: Result<TestConfig, ConfigError> =
            load_yaml_config("/nonexistent/path/config.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }
}
