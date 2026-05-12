//! Integration tests for config-json
//!
//! Tests nested JSON parsing, file round-trip, and struct equality against golden value.

use config_core::{ConfigParser, ValidationError};
use config_json::{JsonParser, JsonValidatable};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
    timeout: u32,
}

impl JsonValidatable for TestConfig {
    fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
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

// Test 1: Nested JSON parsing: 3-level deep struct deserialization
#[test]
fn test_nested_json_parsing() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level3Config {
        value: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level2Config {
        nested: Level3Config,
        count: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Level1Config {
        config: Level2Config,
        enabled: bool,
    }

    impl JsonValidatable for Level1Config {
        fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
            if self.config.count == 0 {
                Err(vec![ValidationError::new(
                    "config.count",
                    "Count must be positive",
                )])
            } else {
                Ok(())
            }
        }
    }

    let json_content = r#"{
  "config": {
    "nested": {
      "value": "deep-value"
    },
    "count": 42
  },
  "enabled": true
}"#;

    let config: Level1Config =
        JsonParser::parse_and_validate(json_content).expect("Should parse nested JSON");

    assert_eq!(config.config.nested.value, "deep-value");
    assert_eq!(config.config.count, 42);
    assert!(config.enabled);
}

// Test 2: File round-trip (tempfile): write JSON, read back, assert equality
#[test]
fn test_file_round_trip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");

    let original_config = TestConfig {
        name: "round-trip-test".to_string(),
        port: 9000,
        timeout: 60,
    };

    // Write config to file
    let json_string = serde_json::to_string_pretty(&original_config).expect("Failed to serialize");
    std::fs::write(&config_path, json_string).expect("Failed to write config file");

    // Read config from file
    let json_string = std::fs::read_to_string(&config_path).expect("Failed to read config file");
    let loaded_config: TestConfig =
        serde_json::from_str(&json_string).expect("Failed to deserialize");

    assert_eq!(
        loaded_config, original_config,
        "Loaded config should match original"
    );

    // Verify the file was actually created
    assert!(config_path.exists(), "Config file should exist");
}

// Test 3: Struct equality against hardcoded expected value (golden-value test)
#[test]
fn test_golden_value() {
    let json_content = r#"{
  "name": "golden-test",
  "port": 8080,
  "timeout": 30
}"#;

    let config: TestConfig =
        JsonParser::parse_and_validate(json_content).expect("Should parse golden value JSON");

    let expected = TestConfig {
        name: "golden-test".to_string(),
        port: 8080,
        timeout: 30,
    };

    assert_eq!(config, expected, "Parsed config should match golden value");
}

// Test 4: Complex JSON with arrays and nested objects
#[test]
fn test_complex_json() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ComplexConfig {
        name: String,
        servers: Vec<Server>,
        metadata: Metadata,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Server {
        host: String,
        port: u16,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Metadata {
        version: String,
        tags: Vec<String>,
    }

    impl JsonValidatable for ComplexConfig {
        fn validate_json(&self) -> Result<(), Vec<ValidationError>> {
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

    let json_content = r#"{
  "name": "complex-app",
  "servers": [
    {
      "host": "server1.example.com",
      "port": 8080
    },
    {
      "host": "server2.example.com",
      "port": 8081
    }
  ],
  "metadata": {
    "version": "1.0.0",
    "tags": ["web", "api", "rust"]
  }
}"#;

    let config: ComplexConfig =
        JsonParser::parse_and_validate(json_content).expect("Should parse complex JSON");

    assert_eq!(config.name, "complex-app");
    assert_eq!(config.servers.len(), 2);
    assert_eq!(config.servers[0].host, "server1.example.com");
    assert_eq!(config.servers[1].port, 8081);
    assert_eq!(config.metadata.version, "1.0.0");
    assert_eq!(config.metadata.tags.len(), 3);
    assert!(config.metadata.tags.contains(&"rust".to_string()));
}

// Test 5: Parse-validate pipeline
#[test]
fn test_parse_validate_pipeline() {
    let json_content = r#"{
  "name": "test-app",
  "port": 8080,
  "timeout": 30
}"#;

    let result = JsonParser::<TestConfig>::parse_and_validate(json_content);
    assert!(
        result.is_ok(),
        "Valid config should parse and validate successfully"
    );
    let config = result.unwrap();
    assert_eq!(config.name, "test-app");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout, 30);
}

// Test 6: Validation errors
#[test]
fn test_validation_errors() {
    let invalid_json = r#"{
  "name": "",
  "port": 0,
  "timeout": 0
}"#;

    let result = JsonParser::<TestConfig>::parse_and_validate(invalid_json);
    assert!(result.is_err(), "Invalid config should fail validation");
    match result {
        Err(config_core::ConfigParseError::Validation(errors)) => {
            assert_eq!(errors.len(), 3, "Should have 3 validation errors");
        }
        _ => panic!("Expected validation error"),
    }
}
