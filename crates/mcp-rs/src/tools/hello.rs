use crate::error::McpResult;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct Hello;

#[derive(Deserialize)]
pub struct Input {
    pub name: String,
}

#[derive(Serialize)]
pub struct Output {
    pub message: String,
}

impl Tool for Hello {
    const NAME: &'static str = "say_hello";
    const DESCRIPTION: &'static str = "Say hello from typed Rust";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        Ok(Output {
            message: format!("Hello, {}! 🦀", input.name),
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_say_hello_basic() {
        let hello = Hello;
        let output = hello
            .run(Input {
                name: "World".to_string(),
            })
            .unwrap();
        assert!(output.message.contains("World"));
        assert!(output.message.contains("Hello"));
    }

    #[test]
    fn test_say_hello_with_special_chars() {
        let hello = Hello;
        let output = hello
            .run(Input {
                name: "Test User 123!".to_string(),
            })
            .unwrap();
        assert!(output.message.contains("Test User 123!"));
    }

    #[test]
    fn test_say_hello_empty_name() {
        let hello = Hello;
        let output = hello
            .run(Input {
                name: "".to_string(),
            })
            .unwrap();
        assert!(output.message.contains("Hello"));
    }

    #[test]
    fn test_schema_has_required_name() {
        let schema = Hello::schema();
        assert_eq!(schema["required"][0], "name");
    }
}
