//! TOML Configuration Parser
//!
//! This crate provides a TOML parser that implements the universal configuration traits
//! from `config-core`. It enables type-safe, zero-cost configuration parsing with support
//! for custom validation logic per type.
//!
//! # Features
//!
//! - **Zero-cost abstraction**: Uses monomorphization for compile-time polymorphism
//! - **Custom validation**: Implement `TomlValidatable` for type-specific validation rules
//! - **Format flexibility**: Can parse any TOML structure into typed Rust structs
//! - **File operations**: Convenience functions for loading and saving TOML files
//!
//! # Example
//!
//! ```rust
//! use config_toml::{TomlParser, TomlValidatable};
//! use config_core::{ConfigParser, ValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! impl TomlValidatable for AppConfig {
//!     fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
//!         let mut errors = Vec::new();
//!         if self.port == 0 {
//!             errors.push(ValidationError::new("port", "Port must be non-zero"));
//!         }
//!         if errors.is_empty() { Ok(()) } else { Err(errors) }
//!     }
//! }
//!
//! let toml_content = r#"
//! name = "my-app"
//! port = 8080
//! "#;
//!
//! let config: AppConfig = TomlParser::parse(toml_content).unwrap();
//! assert_eq!(config.name, "my-app");
//! assert_eq!(config.port, 8080);
//! ```

use config_core::{ConfigError, ConfigParser, ValidationError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// TOML parser implementing universal configuration traits
///
/// Uses compile-time polymorphism through monomorphization to provide
/// zero-cost abstraction. The parser is stateless and works with any type
/// that implements both `Deserialize` and `TomlValidatable`.
///
/// # Type Parameters
///
/// - `T`: The configuration type to parse (must implement `Deserialize` and `TomlValidatable`)
///
/// # Examples
///
/// ```rust
/// use config_toml::TomlParser;
/// use config_core::ConfigParser;
/// use serde::{Deserialize, Serialize};
/// use config_toml::TomlValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { value: String }
///
/// impl TomlValidatable for Config {
///     fn validate_toml(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
/// }
///
/// let parser = TomlParser::<Config>::new();
/// ```
pub struct TomlParser<T> {
    _phantom: PhantomData<T>,
}

impl<T> TomlParser<T> {
    /// Create a new TOML parser for the given type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_toml::TomlParser;
    /// use serde::{Deserialize, Serialize};
    /// use config_toml::TomlValidatable;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct MyConfig;
    ///
    /// impl TomlValidatable for MyConfig {
    ///     fn validate_toml(&self) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
    /// }
    ///
    /// let parser = TomlParser::<MyConfig>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for TomlParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConfigParser<T> for TomlParser<T>
where
    T: for<'de> Deserialize<'de> + Serialize + TomlValidatable,
{
    type Error = TomlConfigError;

    fn parse(input: &str) -> Result<T, Self::Error> {
        toml::from_str(input).map_err(TomlConfigError::Parse)
    }

    fn validate(config: &T) -> Result<(), Vec<ValidationError>> {
        config.validate_toml()
    }
}

/// Trait for types that can be validated when loaded from TOML
///
/// Implementors should validate domain constraints such as valid port ranges,
/// non-empty required fields, etc. Validation logic is separate from parsing
/// for better composability.
///
/// # Examples
///
/// ```rust
/// use config_toml::TomlValidatable;
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
/// impl TomlValidatable for ServerConfig {
///     fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
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
pub trait TomlValidatable {
    /// Validate the configuration object after deserialization
    ///
    /// # Errors
    ///
    /// Returns `Err` with a vector of validation errors if validation fails.
    /// Returns `Ok(())` if the configuration is valid.
    fn validate_toml(&self) -> Result<(), Vec<ValidationError>>;
}

/// Blanket implementation for types that don't need custom validation
///
/// Common Rust types automatically implement `TomlValidatable` with no-op
/// validation. Types can override this by implementing `TomlValidatable` explicitly.
///
/// # Examples
///
/// ```rust
/// use config_toml::TomlValidatable;
/// use serde::{Deserialize, Serialize};
///
/// // No need to implement TomlValidatable manually for simple types
/// #[derive(Serialize, Deserialize)]
/// struct SimpleConfig {
///     count: i32,
///     enabled: bool,
/// }
/// ```
macro_rules! impl_default_validation {
    ($($t:ty),*) => {
        $(
            impl TomlValidatable for $t {
                fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
                    Ok(())
                }
            }
        )*
    };
}

// Common types that don't need validation
impl_default_validation!(String, i32, i64, u32, u64, f32, f64, bool);

/// TOML-specific configuration errors
///
/// Wraps both TOML parsing errors from the `toml` crate and validation errors
/// from our configuration validation logic.
///
/// # Variants
///
/// - `Parse`: Error during TOML parsing (e.g., syntax error, type mismatch)
/// - `Validation`: Configuration validation failures
#[derive(Error, Debug)]
pub enum TomlConfigError {
    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    /// Configuration validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<ValidationError>),
}

impl From<TomlConfigError> for ConfigError {
    fn from(error: TomlConfigError) -> Self {
        match error {
            TomlConfigError::Parse(e) => ConfigError::Parse(e.to_string()),
            TomlConfigError::Validation(errors) => ConfigError::Validation(errors),
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

/// Load and parse TOML configuration from a file
///
/// Convenience function that combines file I/O, parsing, and validation.
/// Automatically validates the parsed configuration.
///
/// # Arguments
///
/// - `path`: File path to the TOML configuration file
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - The file cannot be read (IO error)
/// - The TOML is invalid (parse error)
/// - Validation fails
///
/// # Examples
///
/// ```rust,no_run
/// use config_toml::load_toml_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
/// use config_toml::TomlValidatable;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// impl TomlValidatable for Config {
///     fn validate_toml(&self) -> Result<(), Vec<config_core::ValidationError>> {
///         Ok(())
///     }
/// }
///
/// let config: Config = load_toml_config("config.toml")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn load_toml_config<T>(path: impl AsRef<std::path::Path>) -> Result<T, ConfigError>
where
    T: for<'de> Deserialize<'de> + Serialize + TomlValidatable,
{
    let content = std::fs::read_to_string(path)?;
    let config = TomlParser::parse(&content).map_err(ConfigError::from)?;
    TomlParser::validate(&config).map_err(ConfigError::Validation)?;
    Ok(config)
}

/// Save configuration to a TOML file
///
/// Convenience function that serializes and writes a configuration object
/// to a TOML file. Produces pretty-printed output.
///
/// # Arguments
///
/// - `config`: Configuration object to save
/// - `path`: File path where TOML will be written
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
/// use config_toml::save_toml_config;
/// use config_core::ConfigError;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { port: u16 }
///
/// let config = Config { port: 8080 };
/// save_toml_config(&config, "config.toml")?;
/// # Ok::<(), ConfigError>(())
/// ```
pub fn save_toml_config<T>(config: &T, path: impl AsRef<std::path::Path>) -> Result<(), ConfigError>
where
    T: Serialize,
{
    let content = toml::to_string_pretty(config).map_err(|e| ConfigError::Parse(e.to_string()))?;
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

    impl TomlValidatable for TestConfig {
        fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
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
    fn test_toml_parser_success() {
        let toml_content = r#"
        name = "test-app"
        port = 8080
        enabled = true
        "#;

        let config: TestConfig = TomlParser::parse(toml_content).unwrap();
        assert_eq!(config.name, "test-app");
        assert_eq!(config.port, 8080);
        assert!(config.enabled);
    }

    #[test]
    fn test_toml_parser_validation() {
        let toml_content = r#"
        name = ""
        port = 0
        enabled = true
        "#;

        let config: TestConfig = TomlParser::parse(toml_content).unwrap();
        let validation_result = TomlParser::validate(&config);
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.field == "name"));
        assert!(errors.iter().any(|e| e.field == "port"));
    }

    #[test]
    fn test_toml_parser_parse_and_validate() {
        let toml_content = r#"
        name = "valid-app"
        port = 3000
        enabled = false
        "#;

        let config: TestConfig = TomlParser::parse_and_validate(toml_content).unwrap();
        assert_eq!(config.name, "valid-app");
        assert_eq!(config.port, 3000);
        assert!(!config.enabled);
    }

    #[test]
    fn test_invalid_toml_syntax() {
        let invalid_toml = r#"
        name = test-app"  // Missing quote
        port = 8080
        "#;

        let result: Result<TestConfig, _> = TomlParser::parse(invalid_toml);
        assert!(result.is_err());
        match result.unwrap_err() {
            TomlConfigError::Parse(_) => {}
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_toml_default_validation() {
        // Test blanket implementation for String
        let validation = "test".to_string().validate_toml();
        assert!(validation.is_ok());

        // Test blanket implementation for i32
        let validation = 42i32.validate_toml();
        assert!(validation.is_ok());

        // Test blanket implementation for bool
        let validation = true.validate_toml();
        assert!(validation.is_ok());
    }

    #[test]
    fn test_toml_nested_structures() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct NestedConfig {
            app_name: String,
            server_port: u16,
        }

        impl TomlValidatable for NestedConfig {
            fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
                Ok(())
            }
        }

        let toml_content = r#"
        app_name = "nested-app"
        server_port = 9000
        "#;

        let config: NestedConfig = TomlParser::parse(toml_content).unwrap();
        assert_eq!(config.app_name, "nested-app");
        assert_eq!(config.server_port, 9000);
    }

    #[test]
    fn test_error_conversion_to_config_error() {
        let error = TomlConfigError::Parse(toml::de::Error::custom("test error"));
        let config_error: ConfigError = error.into();
        assert!(config_error.to_string().contains("Parse error"));
    }

    #[test]
    fn test_toml_parser_default() {
        let _parser = TomlParser::<TestConfig>::default();
        // Just verify it constructs successfully
    }

    #[test]
    fn test_toml_blanket_validation_remaining_types() {
        assert!(42u32.validate_toml().is_ok());
        assert!(42u64.validate_toml().is_ok());
        assert!(1.5_f32.validate_toml().is_ok());
        assert!(1.5_f64.validate_toml().is_ok());
        assert!(42i64.validate_toml().is_ok());
    }

    #[test]
    fn test_toml_parse_and_validate_validation_failure() {
        let toml_content = r#"
        name = ""
        port = 0
        enabled = true
        "#;
        let result: Result<TestConfig, _> = TomlParser::parse_and_validate(toml_content);
        assert!(result.is_err());
        match result.unwrap_err() {
            config_core::ConfigParseError::Validation(errors) => {
                assert!(!errors.is_empty());
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_toml_validation_error_display_and_conversion() {
        let errors = vec![ValidationError::new("port", "must be nonzero")];
        let err = TomlConfigError::Validation(errors.clone());
        assert!(err.to_string().contains("Validation failed"));

        let config_err: ConfigError = TomlConfigError::Validation(errors).into();
        assert!(matches!(config_err, ConfigError::Validation(_)));
    }

    #[test]
    fn test_load_and_save_toml_config() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_config_toml_roundtrip.toml");

        let original = TestConfig {
            name: "roundtrip".to_string(),
            port: 4321,
            enabled: false,
        };

        save_toml_config(&original, &path).expect("save failed");

        let loaded: TestConfig = load_toml_config(&path).expect("load failed");
        assert_eq!(loaded.name, "roundtrip");
        assert_eq!(loaded.port, 4321);
        assert!(!loaded.enabled);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_toml_config_file_not_found() {
        let result: Result<TestConfig, ConfigError> =
            load_toml_config("/nonexistent/path/config.toml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }
}
