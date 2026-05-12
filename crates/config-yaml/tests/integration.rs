//! Integration tests for config-yaml
//!
//! Tests parse-validate pipeline, complex YAML features, file round-trip, and error paths.

use config_core::{ConfigParser, ValidationError};
use config_yaml::{load_yaml_config, save_yaml_config, YamlParser, YamlValidatable};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
    timeout: u32,
}

impl YamlValidatable for TestConfig {
    fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
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

// Test 1: Complex YAML features (multiline scalars, nested mappings, sequence-of-maps)
#[test]
fn test_complex_yaml_features() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ComplexConfig {
        app: AppSettings,
        servers: Vec<Server>,
        description: String,
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

    impl YamlValidatable for ComplexConfig {
        fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
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

    let yaml_content = r#"
app:
  name: complex-app
  debug: true

servers:
  - host: server1.example.com
    port: 8080
  - host: server2.example.com
    port: 8081

description: |
  This is a multiline
  description string
  that spans multiple
  lines in YAML.
"#;

    let config: ComplexConfig =
        YamlParser::parse_and_validate(yaml_content).expect("Should parse complex YAML");

    assert_eq!(config.app.name, "complex-app");
    assert!(config.app.debug);
    assert_eq!(config.servers.len(), 2);
    assert_eq!(config.servers[0].host, "server1.example.com");
    assert_eq!(config.servers[1].port, 8081);
    assert!(config.description.contains("multiline"));
}

// Test 2: File round-trip (tempfile): save → reload → struct equality
#[test]
fn test_file_round_trip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let original_config = TestConfig {
        name: "round-trip-test".to_string(),
        port: 9000,
        timeout: 60,
    };

    // Save config to file
    save_yaml_config(&original_config, &config_path).expect("Failed to save config");

    // Load config from file
    let loaded_config =
        load_yaml_config::<TestConfig>(&config_path).expect("Failed to load config");

    assert_eq!(
        loaded_config, original_config,
        "Loaded config should match original"
    );

    // Verify the file was actually created
    assert!(config_path.exists(), "Config file should exist");
}

// Test 3: Error paths: invalid YAML string → ParseError; valid parse but fails YamlValidatable → ValidationError
#[test]
fn test_error_paths() {
    // Test 1: Invalid YAML syntax
    let invalid_yaml = r#"
name: test
port: [unclosed array
"#;

    let result = YamlParser::<TestConfig>::parse_and_validate(invalid_yaml);
    assert!(result.is_err(), "Invalid YAML should fail");
    match result {
        Err(config_core::ConfigParseError::Parse(_)) => {
            // Expected: parse error
        }
        _ => panic!("Expected parse error for invalid YAML"),
    }

    // Test 2: Valid YAML but validation fails
    let valid_yaml_invalid_config = r#"
name: ""
port: 0
timeout: 0
"#;

    let result = YamlParser::<TestConfig>::parse_and_validate(valid_yaml_invalid_config);
    assert!(result.is_err(), "Invalid config should fail validation");
    match result {
        Err(config_core::ConfigParseError::Validation(errors)) => {
            assert_eq!(errors.len(), 3, "Should have 3 validation errors");
        }
        _ => panic!("Expected validation error"),
    }
}

// Test 4: Parse-validate pipeline with YamlValidatable
#[test]
fn test_parse_validate_pipeline() {
    let yaml_content = r#"
name: test-app
port: 8080
timeout: 30
"#;

    let result = YamlParser::<TestConfig>::parse_and_validate(yaml_content);
    assert!(
        result.is_ok(),
        "Valid config should parse and validate successfully"
    );
    let config = result.unwrap();
    assert_eq!(config.name, "test-app");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout, 30);
}

// Test 5: Nested mappings and complex types
#[test]
fn test_nested_mappings() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct NestedConfig {
        database: DatabaseConfig,
        cache: CacheConfig,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DatabaseConfig {
        host: String,
        port: u16,
        name: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct CacheConfig {
        enabled: bool,
        ttl_seconds: u32,
    }

    impl YamlValidatable for NestedConfig {
        fn validate_yaml(&self) -> Result<(), Vec<ValidationError>> {
            if self.database.port == 0 {
                Err(vec![ValidationError::new(
                    "database.port",
                    "Port must be non-zero",
                )])
            } else {
                Ok(())
            }
        }
    }

    let yaml_content = r#"
database:
  host: localhost
  port: 5432
  name: mydb

cache:
  enabled: true
  ttl_seconds: 3600
"#;

    let config: NestedConfig =
        YamlParser::parse_and_validate(yaml_content).expect("Should parse nested YAML");

    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.name, "mydb");
    assert!(config.cache.enabled);
    assert_eq!(config.cache.ttl_seconds, 3600);
}
