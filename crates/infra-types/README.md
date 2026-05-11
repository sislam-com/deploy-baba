# infra-types

Cloud-agnostic infrastructure type definitions for deployment configuration and observability.

## Usage

```ignore
use infra_types::Stack;

let toml_str = r#"
[project]
name = "deploy-baba"
version = "0.1.0"
region = "us-east-1"

[deploy]
mode = "lambda"
function_name = "deploy-baba-ui"
runtime = "provided.al2023"
architecture = "arm64"
memory_mb = 256
timeout_seconds = 30

[database]
path = "/mnt/db/deploy-baba.db"
wal_mode = true

[observability]
log_level = "info"
cloudwatch_namespace = "deploy-baba"

[aws]
profile = "deploy-baba"
state_bucket_prefix = "deploy-baba-tfstate"
ssm_prefix = "/deploy-baba"
"#;

let stack: Stack = toml::from_str(toml_str).expect("valid stack.toml");
assert_eq!(stack.project.name, "deploy-baba");
```

## Features

- Cloud-agnostic design (AWS, GCP, Azure, Local support)
- Type-safe configuration with serde (de)serialization
- SQLite-only database support with optional S3 backups
- Deploy modes: Lambda, ECS Fargate Spot
- Comprehensive error handling via `thiserror`
- TOML-serializable stack configuration
- Observability configuration (CloudWatch, log levels)

## License

MIT
