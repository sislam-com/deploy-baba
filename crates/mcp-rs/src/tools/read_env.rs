use crate::error::McpResult;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

pub struct ReadEnv;

#[derive(Deserialize)]
pub struct Input {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
}

#[derive(Serialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct Output {
    pub variables: Vec<EnvVar>,
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// List of sensitive environment variable patterns to mask
const SENSITIVE_PATTERNS: &[&str] = &[
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "TOKEN",
    "API_KEY",
    "APIKEY",
    "PRIVATE",
    "CREDENTIAL",
    "AUTH",
];

fn is_sensitive(name: &str) -> bool {
    let upper = name.to_uppercase();
    SENSITIVE_PATTERNS
        .iter()
        .any(|pattern| upper.contains(pattern))
}

fn mask_value(name: &str, value: &str) -> String {
    if is_sensitive(name) {
        if value.len() <= 4 {
            "****".to_string()
        } else {
            format!("{}****", &value[..2])
        }
    } else {
        value.to_string()
    }
}

impl Tool for ReadEnv {
    const NAME: &'static str = "read_env";
    const DESCRIPTION: &'static str = "Read environment variables with optional filtering";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        // If a specific name is requested, just return that variable
        if let Some(name) = input.name {
            match env::var(&name) {
                Ok(value) => {
                    return Ok(Output {
                        variables: vec![EnvVar {
                            name: name.clone(),
                            value: mask_value(&name, &value),
                        }],
                        count: 1,
                        error: None,
                    });
                }
                Err(_) => {
                    return Ok(Output {
                        variables: vec![],
                        count: 0,
                        error: Some(format!("Environment variable '{}' not found", name)),
                    });
                }
            }
        }

        // Collect all environment variables
        let mut variables: Vec<EnvVar> = env::vars()
            .filter(|(name, _)| {
                // Filter by prefix if provided
                if let Some(ref prefix) = input.prefix {
                    name.starts_with(prefix)
                } else {
                    true
                }
            })
            .map(|(name, value)| EnvVar {
                value: mask_value(&name, &value),
                name,
            })
            .collect();

        // Sort by name for consistent output
        variables.sort_by(|a, b| a.name.cmp(&b.name));

        let count = variables.len() as u32;

        Ok(Output {
            variables,
            count,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Specific environment variable name to read"
                },
                "prefix": {
                    "type": "string",
                    "description": "Filter variables by name prefix (e.g., 'CARGO', 'PATH')"
                }
            },
            "required": []
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_true() {
        assert!(is_sensitive("MY_SECRET_KEY"));
        assert!(is_sensitive("DATABASE_PASSWORD"));
        assert!(is_sensitive("API_TOKEN"));
        assert!(is_sensitive("AUTH_KEY"));
        assert!(is_sensitive("PRIVATE_DATA"));
    }

    #[test]
    fn test_is_sensitive_false() {
        assert!(!is_sensitive("HOME"));
        assert!(!is_sensitive("PATH"));
        assert!(!is_sensitive("USER"));
        assert!(!is_sensitive("CARGO_HOME"));
    }

    #[test]
    fn test_mask_value_sensitive() {
        // Short value gets fully masked
        assert_eq!(mask_value("MY_SECRET", "abc"), "****");

        // Longer value shows first 2 chars
        assert_eq!(mask_value("API_TOKEN", "mytoken123"), "my****");
    }

    #[test]
    fn test_mask_value_not_sensitive() {
        // Non-sensitive values are not masked
        assert_eq!(mask_value("HOME", "/home/user"), "/home/user");
        assert_eq!(mask_value("PATH", "/usr/bin"), "/usr/bin");
    }

    #[test]
    fn test_read_env_home() {
        // HOME should always exist
        let tool = ReadEnv;
        let output = tool
            .run(Input {
                name: Some("HOME".to_string()),
                prefix: None,
            })
            .unwrap();

        assert_eq!(output.count, 1);
        assert!(output.error.is_none());
        assert_eq!(output.variables[0].name, "HOME");
    }

    #[test]
    fn test_read_env_nonexistent() {
        let tool = ReadEnv;
        let output = tool
            .run(Input {
                name: Some("NONEXISTENT_VAR_XYZ_12345".to_string()),
                prefix: None,
            })
            .unwrap();

        assert_eq!(output.count, 0);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_read_env_with_prefix() {
        let tool = ReadEnv;
        let output = tool
            .run(Input {
                name: None,
                prefix: Some("PATH".to_string()),
            })
            .unwrap();

        // All returned variables should start with PATH
        for var in &output.variables {
            assert!(var.name.starts_with("PATH"));
        }
    }

    #[test]
    fn test_read_env_all() {
        let tool = ReadEnv;
        let output = tool
            .run(Input {
                name: None,
                prefix: None,
            })
            .unwrap();

        // Should return multiple variables
        assert!(output.count > 0);
        assert!(output.error.is_none());

        // Should be sorted alphabetically
        let names: Vec<&str> = output.variables.iter().map(|v| v.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_schema_has_empty_required() {
        let schema = ReadEnv::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
