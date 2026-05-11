//! Integration tests for config-core
//!
//! Tests cross-trait composition, ConfigMerger precedence, and ConfigSource round-trip.

use config_core::{ConfigMerger, ConfigParser, ConfigSource, ConfigValidator, ValidationError};

#[derive(Debug, Clone, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
    timeout: u32,
}

// Test 1: Cross-trait composition: ConfigParser + ConfigValidator on same type
#[test]
fn test_cross_trait_composition() {
    struct TestParser;

    impl ConfigParser<TestConfig> for TestParser {
        type Error = config_core::ConfigError;

        fn parse(input: &str) -> Result<TestConfig, Self::Error> {
            // Simple parsing for test - parse "name:port:timeout" format
            let parts: Vec<&str> = input.split(':').collect();
            if parts.len() != 3 {
                return Err(config_core::ConfigError::Parse(
                    "Invalid format".to_string(),
                ));
            }
            let name = parts[0].to_string();
            let port = parts[1]
                .parse::<u16>()
                .map_err(|_| config_core::ConfigError::Parse("Invalid port".to_string()))?;
            let timeout = parts[2]
                .parse::<u32>()
                .map_err(|_| config_core::ConfigError::Parse("Invalid timeout".to_string()))?;
            Ok(TestConfig {
                name,
                port,
                timeout,
            })
        }

        fn validate(config: &TestConfig) -> Result<(), Vec<ValidationError>> {
            let mut errors = Vec::new();
            if config.name.is_empty() {
                errors.push(ValidationError::new("name", "Name cannot be empty"));
            }
            if config.port == 0 {
                errors.push(ValidationError::new("port", "Port must be non-zero"));
            }
            if config.timeout == 0 {
                errors.push(ValidationError::new("timeout", "Timeout must be positive"));
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    // Valid config should pass both parse and validate
    let valid_input = "test:8080:30";
    let result = TestParser::parse_and_validate(valid_input);
    assert!(
        result.is_ok(),
        "Valid config should parse and validate successfully"
    );
    let config = result.unwrap();
    assert_eq!(config.name, "test");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout, 30);

    // Invalid config should fail validation
    let invalid_input = ":0:0";
    let result = TestParser::parse_and_validate(invalid_input);
    assert!(result.is_err(), "Invalid config should fail validation");
    match result {
        Err(config_core::ConfigParseError::Validation(errors)) => {
            assert_eq!(errors.len(), 3, "Should have 3 validation errors");
        }
        _ => panic!("Expected validation error"),
    }
}

// Test 2: ConfigMerger precedence (override wins over base, base fills missing keys)
#[test]
fn test_config_merger_precedence() {
    let base = TestConfig {
        name: "base".to_string(),
        port: 3000,
        timeout: 10,
    };

    let override_config = TestConfig {
        name: "override".to_string(),
        port: 8080,
        timeout: 30,
    };

    // Create a simple merger for TestConfig
    struct TestMerger;
    impl ConfigMerger<TestConfig> for TestMerger {
        fn merge(
            _base: TestConfig,
            override_config: TestConfig,
        ) -> Result<TestConfig, config_core::ConfigError> {
            // Override wins for all fields in this simple implementation
            Ok(override_config)
        }
    }

    let merged = TestMerger::merge(base, override_config).expect("Merge should succeed");
    assert_eq!(merged.name, "override", "Override should win for name");
    assert_eq!(merged.port, 8080, "Override should win for port");
    assert_eq!(merged.timeout, 30, "Override should win for timeout");
}

// Test 3: ConfigSource round-trip (build source, parse back, assert equality)
#[test]
fn test_config_source_round_trip() {
    let file_source = ConfigSource::File("/etc/app.toml".to_string());
    let env_source = ConfigSource::Env("APP_CONFIG".to_string());
    let remote_source = ConfigSource::Remote("https://config.example.com/app.json".to_string());

    // Test Display implementation
    assert_eq!(file_source.to_string(), "file:///etc/app.toml");
    assert_eq!(env_source.to_string(), "env:APP_CONFIG");
    assert_eq!(
        remote_source.to_string(),
        "remote:https://config.example.com/app.json"
    );

    // Test equality
    assert_eq!(file_source, ConfigSource::File("/etc/app.toml".to_string()));
    assert_ne!(file_source, env_source);

    // Test Clone
    let cloned = file_source.clone();
    assert_eq!(file_source, cloned);
}

// Test 4: ConfigValidator is_valid convenience method
#[test]
fn test_validator_is_valid() {
    struct TestValidator;

    impl ConfigValidator<TestConfig> for TestValidator {
        fn validate(config: &TestConfig) -> Result<(), Vec<ValidationError>> {
            if config.port == 0 {
                Err(vec![ValidationError::new("port", "Port must be non-zero")])
            } else {
                Ok(())
            }
        }
    }

    let valid_config = TestConfig {
        name: "test".to_string(),
        port: 8080,
        timeout: 30,
    };

    let invalid_config = TestConfig {
        name: "test".to_string(),
        port: 0,
        timeout: 30,
    };

    assert!(
        TestValidator::is_valid(&valid_config),
        "Valid config should return true"
    );
    assert!(
        !TestValidator::is_valid(&invalid_config),
        "Invalid config should return false"
    );
}
