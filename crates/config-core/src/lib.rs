//! Universal Configuration Parsing Traits
//!
//! This crate provides universal traits and types for parsing configuration files in any format.
//! It enables zero-cost abstractions where different parsers can be swapped without
//! changing downstream code, following Rust's philosophy of compile-time polymorphism.
//!
//! # Core Concepts
//!
//! - **ConfigParser<T>**: Universal trait for parsing strings into typed configuration objects
//! - **ConfigValidator<T>**: Universal trait for validating configuration objects
//! - **ConfigMerger<T>**: Universal trait for merging multiple configurations
//! - **EnvironmentInterpolator<T>**: Universal trait for environment variable substitution
//! - **ConfigSource**: Enumeration of where configuration comes from
//!
//! # Example: Basic Parsing
//!
//! ```rust
//! use config_core::{ConfigParser, ValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! struct MyParser;
//!
//! impl ConfigParser<AppConfig> for MyParser {
//!     type Error = config_core::ConfigError;
//!
//!     fn parse(input: &str) -> Result<AppConfig, Self::Error> {
//!         // Parse logic here
//!         todo!()
//!     }
//!
//!     fn validate(config: &AppConfig) -> Result<(), Vec<ValidationError>> {
//!         if config.port == 0 {
//!             return Err(vec![ValidationError::new("port", "Port must be non-zero")]);
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Example: With Validation
//!
//! ```rust
//! use config_core::{ConfigParser, ValidationError};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct DatabaseConfig {
//!     host: String,
//!     port: u16,
//!     timeout_ms: u32,
//! }
//!
//! struct DbParser;
//!
//! impl ConfigParser<DatabaseConfig> for DbParser {
//!     type Error = config_core::ConfigError;
//!
//!     fn parse(input: &str) -> Result<DatabaseConfig, Self::Error> {
//!         todo!()
//!     }
//!
//!     fn validate(config: &DatabaseConfig) -> Result<(), Vec<ValidationError>> {
//!         let mut errors = Vec::new();
//!
//!         if config.host.is_empty() {
//!             errors.push(ValidationError::new("host", "Host cannot be empty"));
//!         }
//!
//!         if config.port == 0 {
//!             errors.push(ValidationError::new("port", "Port must be non-zero"));
//!         }
//!
//!         if config.timeout_ms == 0 {
//!             errors.push(ValidationError::new("timeout_ms", "Timeout must be positive"));
//!         }
//!
//!         if errors.is_empty() { Ok(()) } else { Err(errors) }
//!     }
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Source of configuration data
///
/// Indicates where a configuration value originated, enabling intelligent
/// merging and conflict resolution strategies.
///
/// # Examples
///
/// ```rust
/// use config_core::ConfigSource;
///
/// let file_source = ConfigSource::File("/etc/app.toml".into());
/// let env_source = ConfigSource::Env("APP_CONFIG".into());
/// let remote_source = ConfigSource::Remote("https://config.example.com/app.json".into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// Configuration loaded from a local file
    File(String),

    /// Configuration loaded from environment variables
    Env(String),

    /// Configuration loaded from a remote source (HTTP, S3, etc.)
    Remote(String),
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigSource::File(path) => write!(f, "file://{}", path),
            ConfigSource::Env(name) => write!(f, "env:{}", name),
            ConfigSource::Remote(url) => write!(f, "remote:{}", url),
        }
    }
}

/// Universal trait for parsing configuration from string input
///
/// Implementations provide format-specific parsing logic while maintaining
/// a consistent interface. This enables zero-cost polymorphism through
/// monomorphization.
///
/// # Associated Types
///
/// - `Error`: Must implement `std::error::Error + Send + Sync + 'static`
///
/// # Examples
///
/// ```rust
/// use config_core::ConfigParser;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { value: i32 }
///
/// struct CustomParser;
///
/// impl ConfigParser<Config> for CustomParser {
///     type Error = config_core::ConfigError;
///
///     fn parse(input: &str) -> Result<Config, Self::Error> {
///         // Parse implementation
///         todo!()
///     }
///
///     fn validate(_config: &Config) -> Result<(), Vec<config_core::ValidationError>> {
///         Ok(())
///     }
/// }
/// ```
pub trait ConfigParser<T> {
    /// The error type returned by this parser
    type Error: std::error::Error + Send + Sync + 'static;

    /// Parse configuration from string input
    ///
    /// # Errors
    ///
    /// Returns `Err` if the input cannot be parsed into type `T`.
    fn parse(input: &str) -> Result<T, Self::Error>;

    /// Validate parsed configuration
    ///
    /// # Errors
    ///
    /// Returns `Err(Vec<ValidationError>)` if validation fails.
    /// The vector contains details about each validation error.
    fn validate(config: &T) -> Result<(), Vec<ValidationError>>;

    /// Parse and validate in one operation
    ///
    /// Combines parsing and validation to provide clear error reporting
    /// that distinguishes between parse-time and validation-time failures.
    ///
    /// # Errors
    ///
    /// Returns `ConfigParseError::Parse` if parsing fails, or
    /// `ConfigParseError::Validation` if validation fails.
    fn parse_and_validate(input: &str) -> Result<T, ConfigParseError<Self::Error>> {
        let config = Self::parse(input).map_err(ConfigParseError::Parse)?;
        Self::validate(&config).map_err(ConfigParseError::Validation)?;
        Ok(config)
    }
}

/// Universal trait for configuration validation
///
/// Provides validation operations that can be implemented separately
/// from parsing for better composition and testability.
///
/// # Examples
///
/// ```rust
/// use config_core::{ConfigValidator, ValidationError};
///
/// struct MyValidator;
/// struct Config { port: u16 }
///
/// impl ConfigValidator<Config> for MyValidator {
///     fn validate(config: &Config) -> Result<(), Vec<ValidationError>> {
///         if config.port == 0 {
///             Err(vec![ValidationError::new("port", "Port must be non-zero")])
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait ConfigValidator<T> {
    /// Validate a configuration object
    ///
    /// # Errors
    ///
    /// Returns `Err(Vec<ValidationError>)` if any validation rules fail.
    fn validate(config: &T) -> Result<(), Vec<ValidationError>>;

    /// Check if configuration is valid (convenience method)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_core::ConfigValidator;
    /// # struct MyValidator;
    /// # struct Config;
    /// # impl ConfigValidator<Config> for MyValidator {
    /// #     fn validate(_: &Config) -> Result<(), Vec<config_core::ValidationError>> { Ok(()) }
    /// # }
    /// let config = Config;
    /// let is_valid = MyValidator::is_valid(&config);
    /// ```
    fn is_valid(config: &T) -> bool {
        Self::validate(config).is_ok()
    }
}

/// Validation error with field and message
///
/// Represents a single validation failure with contextual information.
///
/// # Examples
///
/// ```rust
/// use config_core::ValidationError;
///
/// let error = ValidationError::new("port", "Port must be between 1 and 65535");
/// assert_eq!(error.field, "port");
/// assert!(error.message.contains("between"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The field that failed validation
    pub field: String,
    /// Human-readable error message
    pub message: String,
}

impl ValidationError {
    /// Create a new validation error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_core::ValidationError;
    ///
    /// let error = ValidationError::new("port", "Port must be positive");
    /// assert_eq!(error.field, "port");
    /// ```
    pub fn new<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validation error in field '{}': {}",
            self.field, self.message
        )
    }
}

/// Combined error type for parsing and validation
///
/// Distinguishes between errors that occur during the parsing phase
/// versus the validation phase, enabling better error handling and recovery.
///
/// # Variants
///
/// - `Parse(E)`: Error during the parsing phase
/// - `Validation(Vec<ValidationError>)`: One or more validation errors
///
/// # Examples
///
/// ```rust,no_run
/// use config_core::{ConfigParser, ConfigParseError};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config { value: i32 }
///
/// struct TestParser;
///
/// impl ConfigParser<Config> for TestParser {
///     type Error = config_core::ConfigError;
///     fn parse(input: &str) -> Result<Config, Self::Error> { todo!() }
///     fn validate(_: &Config) -> Result<(), Vec<config_core::ValidationError>> { todo!() }
/// }
///
/// // Usage:
/// let result: Result<Config, ConfigParseError<_>> = TestParser::parse_and_validate("{}");
/// ```
#[derive(Error, Debug)]
pub enum ConfigParseError<E: std::error::Error> {
    /// Error during parsing phase
    #[error("Parse error: {0}")]
    Parse(E),

    /// Error during validation phase
    #[error("Validation errors: {}", format_validation_errors(.0))]
    Validation(Vec<ValidationError>),
}

/// Configuration-related errors
///
/// Covers IO errors, parsing errors, validation failures, and unsupported formats.
///
/// # Variants
///
/// - `Io`: File system or IO errors
/// - `Parse`: Format-specific parsing errors
/// - `Validation`: Configuration validation errors
/// - `UnsupportedFormat`: The requested format is not supported
///
/// # Examples
///
/// ```rust
/// use config_core::ConfigError;
/// use std::io;
///
/// let io_err = ConfigError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
/// let parse_err = ConfigError::Parse("Invalid TOML".to_string());
/// ```
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error from underlying format parser
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation failed
    #[error("Validation failed: {}", format_validation_errors(.0))]
    Validation(Vec<ValidationError>),

    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Helper function to format validation errors for display
fn format_validation_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(|e| format!("{}", e))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Trait for configuration formats that support environment variable interpolation
///
/// Enables dynamic substitution of environment variables within configuration
/// values. For example, replacing `${DB_PASSWORD}` with the actual environment variable.
///
/// # Examples
///
/// ```rust
/// use config_core::EnvironmentInterpolator;
///
/// struct Config {
///     database_url: String,
/// }
///
/// struct EnvInterpolator;
///
/// impl EnvironmentInterpolator<Config> for EnvInterpolator {
///     fn interpolate_env(config: &mut Config) -> Result<(), config_core::ConfigError> {
///         // Replace ${VAR_NAME} with actual environment variable values
///         Ok(())
///     }
/// }
/// ```
pub trait EnvironmentInterpolator<T> {
    /// Interpolate environment variables in the configuration
    ///
    /// # Errors
    ///
    /// Returns error if a referenced environment variable doesn't exist
    /// or if interpolation fails.
    fn interpolate_env(config: &mut T) -> Result<(), ConfigError>;
}

/// Trait for configuration formats that support merging
///
/// Enables combining configurations from multiple sources with predictable
/// precedence rules. Typically, values from `other` take precedence over `base`.
///
/// # Examples
///
/// ```rust
/// use config_core::ConfigMerger;
///
/// #[derive(Debug)]
/// struct Config { value: Option<i32> }
///
/// struct ConfigMergeImpl;
///
/// impl ConfigMerger<Config> for ConfigMergeImpl {
///     fn merge(base: Config, other: Config) -> Result<Config, config_core::ConfigError> {
///         Ok(Config {
///             value: other.value.or(base.value),
///         })
///     }
/// }
/// ```
pub trait ConfigMerger<T> {
    /// Merge two configurations, with `other` taking precedence
    ///
    /// # Errors
    ///
    /// Returns error if merging is not possible or leads to invalid state.
    fn merge(base: T, other: T) -> Result<T, ConfigError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    struct MockParser;

    impl ConfigParser<TestConfig> for MockParser {
        type Error = ConfigError;

        fn parse(input: &str) -> Result<TestConfig, Self::Error> {
            if input == "valid config" {
                Ok(TestConfig {
                    name: "test".to_string(),
                    port: 8080,
                })
            } else {
                Err(ConfigError::Parse("Invalid input".to_string()))
            }
        }

        fn validate(config: &TestConfig) -> Result<(), Vec<ValidationError>> {
            let mut errors = Vec::new();
            if config.port == 0 {
                errors.push(ValidationError::new("port", "Port cannot be zero"));
            }
            if config.name.is_empty() {
                errors.push(ValidationError::new("name", "Name cannot be empty"));
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn test_config_parser_success() {
        let result = MockParser::parse_and_validate("valid config");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_parser_parse_error() {
        let result = MockParser::parse_and_validate("invalid config");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigParseError::Parse(_) => {}
            ConfigParseError::Validation(_) => panic!("Expected parse error, got validation error"),
        }
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::new("port", "Port must be positive");
        assert_eq!(
            error.to_string(),
            "Validation error in field 'port': Port must be positive"
        );
    }

    #[test]
    fn test_config_source_display() {
        assert_eq!(
            ConfigSource::File("/etc/app.toml".to_string()).to_string(),
            "file:///etc/app.toml"
        );
        assert_eq!(
            ConfigSource::Env("APP_CONFIG".to_string()).to_string(),
            "env:APP_CONFIG"
        );
        assert_eq!(
            ConfigSource::Remote("https://config.example.com/app.json".to_string()).to_string(),
            "remote:https://config.example.com/app.json"
        );
    }

    #[test]
    fn test_validation_error_new() {
        let err = ValidationError::new("field", "message");
        assert_eq!(err.field, "field");
        assert_eq!(err.message, "message");
    }

    #[test]
    fn test_multiple_validation_errors() {
        let errors = [
            ValidationError::new("field1", "error1"),
            ValidationError::new("field2", "error2"),
        ];
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_parse_and_validate_validation_failure() {
        let config = TestConfig {
            name: "test".to_string(),
            port: 0,
        };
        let errors = MockParser::validate(&config).unwrap_err();
        assert!(errors.iter().any(|e| e.field == "port"));

        let result = MockParser::parse_and_validate("valid config");
        assert!(result.is_ok());

        struct ZeroPortParser;
        impl ConfigParser<TestConfig> for ZeroPortParser {
            type Error = ConfigError;
            fn parse(_: &str) -> Result<TestConfig, Self::Error> {
                Ok(TestConfig {
                    name: "x".to_string(),
                    port: 0,
                })
            }
            fn validate(config: &TestConfig) -> Result<(), Vec<ValidationError>> {
                if config.port == 0 {
                    Err(vec![ValidationError::new("port", "zero")])
                } else {
                    Ok(())
                }
            }
        }
        let result = ZeroPortParser::parse_and_validate("anything");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigParseError::Validation(_)
        ));
    }

    #[test]
    fn test_config_validator_is_valid() {
        struct PortValidator;
        impl ConfigValidator<TestConfig> for PortValidator {
            fn validate(config: &TestConfig) -> Result<(), Vec<ValidationError>> {
                if config.port == 0 {
                    Err(vec![ValidationError::new("port", "zero")])
                } else {
                    Ok(())
                }
            }
        }

        let valid = TestConfig {
            name: "x".to_string(),
            port: 8080,
        };
        let invalid = TestConfig {
            name: "x".to_string(),
            port: 0,
        };

        assert!(PortValidator::is_valid(&valid));
        assert!(!PortValidator::is_valid(&invalid));
    }

    #[test]
    fn test_config_error_variants_display() {
        let errors = vec![ValidationError::new("f", "m")];
        let err = ConfigError::Validation(errors);
        assert!(err.to_string().contains("Validation failed"));

        let err = ConfigError::UnsupportedFormat("toml".to_string());
        assert!(err.to_string().contains("Unsupported format"));

        let err = ConfigError::Parse("bad input".to_string());
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_config_merger_trait() {
        #[derive(Debug)]
        struct OverrideConfig {
            value: Option<i32>,
        }

        struct Merger;
        impl ConfigMerger<OverrideConfig> for Merger {
            fn merge(
                base: OverrideConfig,
                other: OverrideConfig,
            ) -> Result<OverrideConfig, ConfigError> {
                Ok(OverrideConfig {
                    value: other.value.or(base.value),
                })
            }
        }

        let base = OverrideConfig { value: Some(1) };
        let other = OverrideConfig { value: Some(2) };
        let merged = Merger::merge(base, other).unwrap();
        assert_eq!(merged.value, Some(2));

        let base2 = OverrideConfig { value: Some(1) };
        let other2 = OverrideConfig { value: None };
        let merged2 = Merger::merge(base2, other2).unwrap();
        assert_eq!(merged2.value, Some(1));
    }

    #[test]
    fn test_environment_interpolator_trait() {
        struct Config {
            url: String,
        }
        struct Interpolator;
        impl EnvironmentInterpolator<Config> for Interpolator {
            fn interpolate_env(config: &mut Config) -> Result<(), ConfigError> {
                config.url = config.url.replace("${HOST}", "localhost");
                Ok(())
            }
        }

        let mut config = Config {
            url: "${HOST}:8080".to_string(),
        };
        Interpolator::interpolate_env(&mut config).unwrap();
        assert_eq!(config.url, "localhost:8080");
    }
}
