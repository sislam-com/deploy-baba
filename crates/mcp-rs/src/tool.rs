use crate::error::McpResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait Tool {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;

    type Input: for<'de> Deserialize<'de>;
    type Output: Serialize;

    fn run(&self, input: Self::Input) -> McpResult<Self::Output>;
    fn schema() -> Value;
}
