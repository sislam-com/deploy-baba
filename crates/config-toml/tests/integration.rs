//! Integration tests for config-toml
//!
//! Tests parse-validate pipeline, file round-trip, and cross-crate trait usage.

use config_core::{ConfigParser, ValidationError};
use config_toml::{load_toml_config, save_toml_config, TomlParser, TomlValidatable};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
    timeout: u32,
}

impl TomlValidatable for TestConfig {
    fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push(ValidationError::new("name", "Name cannot be empty"));
        }
        if self.port == 0 {
            errors.push(ValidationError::new("port", "Port must be non-zero"));
        }
        if self.timeout == 0 {
            errors.push(ValidationError::new("timeout", "Timeout must be positive"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Test 1: Parse-validate pipeline with TomlValidatable (valid → passes)
#[test]
fn test_parse_validate_valid() {
    let toml_content = r#"
name = "test-app"
port = 8080
timeout = 30
"#;

    let result = TomlParser::<TestConfig>::parse_and_validate(toml_content);
    assert!(
        result.is_ok(),
        "Valid config should parse and validate successfully"
    );
    let config = result.unwrap();
    assert_eq!(config.name, "test-app");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout, 30);
}

// Test 2: Parse-validate pipeline with TomlValidatable (invalid → ValidationError)
#[test]
fn test_parse_validate_invalid() {
    let invalid_toml = r#"
name = ""
port = 0
timeout = 0
"#;

    let result = TomlParser::<TestConfig>::parse_and_validate(invalid_toml);
    assert!(result.is_err(), "Invalid config should fail validation");
    match result {
        Err(config_core::ConfigParseError::Validation(errors)) => {
            assert_eq!(errors.len(), 3, "Should have 3 validation errors");
            let error_fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
            assert!(error_fields.contains(&"name"), "Should have name error");
            assert!(error_fields.contains(&"port"), "Should have port error");
            assert!(
                error_fields.contains(&"timeout"),
                "Should have timeout error"
            );
        }
        _ => panic!("Expected validation error"),
    }
}

// Test 3: File round-trip via save_toml_config / load_toml_config
#[test]
fn test_file_round_trip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");

    let original_config = TestConfig {
        name: "round-trip-test".to_string(),
        port: 9000,
        timeout: 60,
    };

    // Save config to file
    save_toml_config(&original_config, &config_path).expect("Failed to save config");

    // Load config from file
    let loaded_config =
        load_toml_config::<TestConfig>(&config_path).expect("Failed to load config");

    assert_eq!(
        loaded_config, original_config,
        "Loaded config should match original"
    );

    // Verify the file was actually created
    assert!(config_path.exists(), "Config file should exist");
}

// Test 4: Cross-crate trait usage - implement ConfigParser for local test struct
#[test]
fn test_cross_crate_trait_usage() {
    // This test demonstrates that TomlParser implements the universal ConfigParser trait
    // from config-core, enabling cross-crate trait composition

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct LocalConfig {
        app_name: String,
        max_connections: u32,
    }

    impl TomlValidatable for LocalConfig {
        fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
            if self.max_connections == 0 {
                Err(vec![ValidationError::new(
                    "max_connections",
                    "Must be at least 1",
                )])
            } else {
                Ok(())
            }
        }
    }

    let toml_content = r#"
app_name = "local-test"
max_connections = 100
"#;

    // Use the universal ConfigParser trait from config-core
    let config: LocalConfig =
        TomlParser::parse_and_validate(toml_content).expect("Should parse and validate");

    assert_eq!(config.app_name, "local-test");
    assert_eq!(config.max_connections, 100);

    // Test validation failure
    let invalid_toml = r#"
app_name = "local-test"
max_connections = 0
"#;

    let result = TomlParser::<LocalConfig>::parse_and_validate(invalid_toml);
    assert!(result.is_err(), "Should fail validation");
}

// Test 5: Complex TOML features (nested structures, arrays)
#[test]
fn test_complex_toml_features() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ComplexConfig {
        app: AppSettings,
        servers: Vec<Server>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct AppSettings {
        name: String,
        debug: bool,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Server {
        host: String,
        port: u16,
    }

    impl TomlValidatable for ComplexConfig {
        fn validate_toml(&self) -> Result<(), Vec<ValidationError>> {
            if self.servers.is_empty() {
                Err(vec![ValidationError::new(
                    "servers",
                    "Must have at least one server",
                )])
            } else {
                Ok(())
            }
        }
    }

    let toml_content = r#"
[app]
name = "complex-app"
debug = true

[[servers]]
host = "server1.example.com"
port = 8080

[[servers]]
host = "server2.example.com"
port = 8081
"#;

    let config: ComplexConfig =
        TomlParser::parse_and_validate(toml_content).expect("Should parse complex TOML");

    assert_eq!(config.app.name, "complex-app");
    assert_eq!(config.app.debug, true);
    assert_eq!(config.servers.len(), 2);
    assert_eq!(config.servers[0].host, "server1.example.com");
    assert_eq!(config.servers[1].port, 8081);
}
