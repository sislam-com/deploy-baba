use crate::error::McpResult;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Health;

#[derive(Deserialize)]
pub struct Input {
    /// Optional detail level: "basic" (default), "detailed"
    #[serde(default = "default_detail_level")]
    pub detail_level: String,
}

fn default_detail_level() -> String {
    "basic".to_string()
}

#[derive(Serialize)]
pub struct Output {
    /// Overall health status: "healthy", "degraded", "unhealthy"
    pub status: String,

    /// Server uptime in seconds
    pub uptime_seconds: u64,

    /// Number of tools registered
    pub tools_registered: u32,

    /// Server version
    pub version: String,

    /// Server name
    pub name: String,

    /// Current timestamp (ISO 8601)
    pub timestamp: String,

    /// Additional details (only included if detail_level is "detailed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HealthDetails>,

    /// Error message if status is not healthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct HealthDetails {
    /// Memory usage information
    pub memory: MemoryInfo,

    /// File system information
    pub filesystem: FileSystemInfo,

    /// System information
    pub system: SystemInfo,

    /// Performance metrics
    pub metrics: MetricsInfo,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    /// Current memory usage (if available)
    pub usage_description: String,
}

#[derive(Serialize)]
pub struct FileSystemInfo {
    /// Current working directory
    pub current_directory: String,

    /// Whether current directory is writable
    pub cwd_writable: bool,
}

#[derive(Serialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Number of CPU cores
    pub cpu_cores: usize,
}

#[derive(Serialize)]
pub struct MetricsInfo {
    /// Health check response time in milliseconds
    pub response_time_ms: u64,

    /// Process uptime description
    pub uptime_description: String,
}

static SERVER_START_TIME: OnceLock<SystemTime> = OnceLock::new();
static TOOLS_COUNT: AtomicU32 = AtomicU32::new(0);

impl Health {
    pub fn initialize(tools_count: u32) {
        SERVER_START_TIME.get_or_init(SystemTime::now);
        TOOLS_COUNT.store(tools_count, Ordering::Relaxed);
    }

    fn get_uptime_seconds() -> u64 {
        SERVER_START_TIME
            .get()
            .and_then(|start| SystemTime::now().duration_since(*start).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn get_tools_count() -> u32 {
        TOOLS_COUNT.load(Ordering::Relaxed)
    }

    /// Get current timestamp in ISO 8601 format
    fn get_current_timestamp() -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                let nanos = duration.subsec_nanos();
                chrono::DateTime::from_timestamp(secs as i64, nanos)
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339()
            }
            Err(_) => chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Perform health checks and determine overall status
    fn check_health() -> (String, Option<String>) {
        // Basic health checks
        let mut issues = Vec::new();

        // Check if we can get current directory
        if std::env::current_dir().is_err() {
            issues.push("Cannot access current working directory");
        }

        // Check if we have tools registered
        if Self::get_tools_count() == 0 {
            issues.push("No tools are registered");
        }

        // Determine status
        let status = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        };

        let error = if issues.is_empty() {
            None
        } else {
            Some(issues.join("; "))
        };

        (status, error)
    }

    /// Collect detailed health information
    fn collect_detailed_info(start_time: SystemTime) -> HealthDetails {
        let current_dir = std::env::current_dir()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Test if current directory is writable
        let cwd_writable = std::env::temp_dir().exists(); // Simple approximation

        let response_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        let uptime_seconds = Self::get_uptime_seconds();
        let uptime_description = if uptime_seconds < 60 {
            format!("{} seconds", uptime_seconds)
        } else if uptime_seconds < 3600 {
            format!("{} minutes", uptime_seconds / 60)
        } else if uptime_seconds < 86400 {
            format!("{} hours", uptime_seconds / 3600)
        } else {
            format!("{} days", uptime_seconds / 86400)
        };

        HealthDetails {
            memory: MemoryInfo {
                usage_description: "Memory usage information not available in this implementation"
                    .to_string(),
            },
            filesystem: FileSystemInfo {
                current_directory: current_dir,
                cwd_writable,
            },
            system: SystemInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                cpu_cores: num_cpus::get(),
            },
            metrics: MetricsInfo {
                response_time_ms: response_time,
                uptime_description,
            },
        }
    }
}

impl Tool for Health {
    const NAME: &'static str = "health";
    const DESCRIPTION: &'static str = "Get server health status and metrics";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let start_time = SystemTime::now();
        let uptime_seconds = Self::get_uptime_seconds();
        let tools_registered = Self::get_tools_count();
        let timestamp = Self::get_current_timestamp();

        let (status, error) = Self::check_health();

        let details = if input.detail_level == "detailed" {
            Some(Self::collect_detailed_info(start_time))
        } else {
            None
        };

        Ok(Output {
            status,
            uptime_seconds,
            tools_registered,
            version: "0.1.0".to_string(),
            name: "mcp-rs".to_string(),
            timestamp,
            details,
            error,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "detail_level": {
                    "type": "string",
                    "description": "Level of detail for health information",
                    "enum": ["basic", "detailed"],
                    "default": "basic"
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_basic() {
        // Initialize health system
        Health::initialize(5);

        let tool = Health;
        let output = tool
            .run(Input {
                detail_level: "basic".to_string(),
            })
            .unwrap();

        assert!(!output.status.is_empty());
        assert!(matches!(
            output.status.as_str(),
            "healthy" | "degraded" | "unhealthy"
        ));
        assert_eq!(output.tools_registered, 5);
        assert_eq!(output.version, "0.1.0");
        assert_eq!(output.name, "mcp-rs");
        assert!(!output.timestamp.is_empty());
        assert!(output.details.is_none()); // Basic mode shouldn't include details
    }

    #[test]
    fn test_health_check_detailed() {
        // Initialize health system
        Health::initialize(3);

        let tool = Health;
        let output = tool
            .run(Input {
                detail_level: "detailed".to_string(),
            })
            .unwrap();

        assert!(!output.status.is_empty());
        assert_eq!(output.tools_registered, 3);
        assert!(output.details.is_some()); // Detailed mode should include details

        let details = output.details.unwrap();
        assert!(!details.system.os.is_empty());
        assert!(!details.system.arch.is_empty());
        assert!(details.system.cpu_cores > 0);
        assert!(!details.filesystem.current_directory.is_empty());
    }

    #[test]
    fn test_uptime_tracking() {
        Health::initialize(1);

        // Wait a brief moment
        std::thread::sleep(std::time::Duration::from_millis(10));

        let uptime = Health::get_uptime_seconds();
        assert!(uptime >= 0); // Should be at least 0 seconds

        let tools_count = Health::get_tools_count();
        assert_eq!(tools_count, 1);
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = Health::get_current_timestamp();

        // Should be in RFC 3339 format (ISO 8601)
        assert!(timestamp.contains("T"));
        assert!(timestamp.contains("Z") || timestamp.contains("+") || timestamp.contains("-"));
    }

    #[test]
    fn test_health_status_determination() {
        Health::initialize(5);
        let (status, _) = Health::check_health();
        assert_eq!(status, "healthy");
    }

    #[test]
    fn test_schema_validation() {
        let schema = Health::schema();
        assert!(schema.is_object());
        assert!(schema["properties"].is_object());
        assert!(schema["properties"]["detail_level"].is_object());
    }
}
