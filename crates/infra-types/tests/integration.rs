//! Integration tests for infra-types
//!
//! Tests cross-module composition, TOML round-trip serialization,
//! and builder patterns across the Stack and its components.

use infra_types::{
    AlertConfig, AwsConfig, DeployConfig, LogLevel, MetricsConfig, ObservabilityConfig,
    ProjectConfig, S3BackupConfig, SqliteConfig, Stack,
};

// Test 1: Stack TOML round-trip serialization
#[test]
fn test_stack_toml_round_trip() {
    let original = Stack {
        project: ProjectConfig {
            name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            region: "us-east-1".to_string(),
        },
        deploy: DeployConfig {
            mode: "lambda".to_string(),
            function_name: "test-func".to_string(),
            runtime: "provided.al2023".to_string(),
            architecture: "arm64".to_string(),
            memory_mb: 512,
            timeout_seconds: 60,
        },
        database: SqliteConfig {
            path: "/mnt/db/test.db".to_string(),
            wal_mode: true,
            backup: Some(S3BackupConfig {
                bucket: "backups".to_string(),
                prefix: Some("test/".to_string()),
                retain_versions: 30,
                schedule: "rate(1 day)".to_string(),
            }),
        },
        observability: ObservabilityConfig {
            log_level: LogLevel::Debug,
            metrics: Some(MetricsConfig {
                namespace: "test-app".to_string(),
                enabled: true,
            }),
            alerts: Some(AlertConfig {
                email: Some("admin@example.com".to_string()),
                sns_topic_arn: None,
            }),
        },
        aws: AwsConfig {
            profile: "test-profile".to_string(),
            state_bucket_prefix: "test-tfstate".to_string(),
            ssm_prefix: "/test-app".to_string(),
        },
    };

    let toml_str = toml::to_string(&original).expect("Should serialize to TOML");
    let deserialized: Stack = toml::from_str(&toml_str).expect("Should deserialize from TOML");

    assert_eq!(deserialized.project.name, original.project.name);
    assert_eq!(deserialized.project.version, original.project.version);
    assert_eq!(deserialized.project.region, original.project.region);
    assert_eq!(deserialized.deploy.mode, original.deploy.mode);
    assert_eq!(deserialized.deploy.memory_mb, original.deploy.memory_mb);
    assert_eq!(deserialized.database.path, original.database.path);
    assert!(deserialized.database.backup.is_some());
    assert_eq!(
        deserialized.observability.log_level,
        original.observability.log_level
    );
    assert!(deserialized.observability.metrics.is_some());
    assert!(deserialized.observability.alerts.is_some());
    assert_eq!(deserialized.aws.profile, original.aws.profile);
}

// Test 2: Stack with minimal configuration (defaults)
#[test]
fn test_stack_minimal_config() {
    let stack = Stack {
        project: ProjectConfig {
            name: "minimal-app".to_string(),
            version: "0.1.0".to_string(),
            region: "us-west-2".to_string(),
        },
        deploy: DeployConfig {
            mode: "lambda".to_string(),
            function_name: "minimal-func".to_string(),
            runtime: "provided.al2023".to_string(),
            architecture: "arm64".to_string(),
            memory_mb: 128,
            timeout_seconds: 30,
        },
        database: SqliteConfig::default(),
        observability: ObservabilityConfig::default(),
        aws: AwsConfig::default(),
    };

    assert_eq!(stack.project.name, "minimal-app");
    assert_eq!(stack.database.path, "/mnt/db/app.db");
    assert!(stack.database.wal_mode);
    assert_eq!(stack.observability.log_level, LogLevel::Info);
    assert!(stack.observability.metrics.is_none());
    assert!(stack.observability.alerts.is_none());
    assert_eq!(stack.aws.profile, "default");
}

// Test 3: Cross-module builder pattern composition
#[test]
fn test_cross_module_builder_composition() {
    let project = ProjectConfig::new("builder-app", "2.0.0", "eu-west-1");
    let backup = S3BackupConfig::new("app-backups", "db/").with_retain_versions(14);
    let database = SqliteConfig::with_path("/data/app.db")
        .with_wal_mode(true)
        .with_backup(backup);
    let metrics = MetricsConfig {
        namespace: "builder-app".to_string(),
        enabled: true,
    };
    let observability = ObservabilityConfig {
        log_level: LogLevel::Trace,
        metrics: Some(metrics),
        alerts: None,
    };
    let aws = AwsConfig::new("builder-profile", "builder-tfstate", "/builder");

    let stack = Stack {
        project,
        deploy: DeployConfig {
            mode: "lambda".to_string(),
            function_name: "builder-func".to_string(),
            runtime: "provided.al2023".to_string(),
            architecture: "arm64".to_string(),
            memory_mb: 256,
            timeout_seconds: 45,
        },
        database,
        observability,
        aws,
    };

    assert_eq!(stack.project.name, "builder-app");
    assert_eq!(stack.database.backup.as_ref().unwrap().retain_versions, 14);
    assert_eq!(stack.observability.log_level, LogLevel::Trace);
    assert!(stack.observability.metrics.as_ref().unwrap().enabled);
    assert_eq!(stack.aws.profile, "builder-profile");
}

// Test 4: Stack identifier method
#[test]
fn test_stack_identifier() {
    let stack = Stack {
        project: ProjectConfig {
            name: "id-test".to_string(),
            version: "1.0.0".to_string(),
            region: "ap-northeast-1".to_string(),
        },
        deploy: DeployConfig {
            mode: "lambda".to_string(),
            function_name: "id-func".to_string(),
            runtime: "provided.al2023".to_string(),
            architecture: "arm64".to_string(),
            memory_mb: 256,
            timeout_seconds: 30,
        },
        database: SqliteConfig::default(),
        observability: ObservabilityConfig::default(),
        aws: AwsConfig::default(),
    };

    assert_eq!(stack.identifier(), "id-test-ap-northeast-1");
}

// Test 5: DeployConfig helper methods
#[test]
fn test_deploy_config_helpers() {
    let lambda_config = DeployConfig {
        mode: "lambda".to_string(),
        function_name: "lambda-func".to_string(),
        runtime: "provided.al2023".to_string(),
        architecture: "arm64".to_string(),
        memory_mb: 256,
        timeout_seconds: 30,
    };

    assert!(lambda_config.is_lambda());
    assert!(!lambda_config.is_ecs_fargate_spot());
    assert!(lambda_config.is_arm64());

    let ecs_config = DeployConfig {
        mode: "ecs-fargate-spot".to_string(),
        function_name: "ecs-func".to_string(),
        runtime: "provided.al2023".to_string(),
        architecture: "x86_64".to_string(),
        memory_mb: 2048,
        timeout_seconds: 300,
    };

    assert!(!ecs_config.is_lambda());
    assert!(ecs_config.is_ecs_fargate_spot());
    assert!(!ecs_config.is_arm64());
}
