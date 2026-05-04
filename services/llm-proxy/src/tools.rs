use llm_core::ToolDef;

pub fn portfolio_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "list_jobs".to_string(),
            description: "List all job positions with company, title, dates, and tech stack"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDef {
            name: "get_job_details".to_string(),
            description: "Get details for a specific job position including accomplishments"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "slug": {
                        "type": "string",
                        "description": "The job slug, e.g. 'scala-computing'"
                    }
                },
                "required": ["slug"]
            }),
        },
        ToolDef {
            name: "list_competencies".to_string(),
            description: "List all competency categories with descriptions and evidence highlights"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDef {
            name: "get_about".to_string(),
            description: "Get about sections describing the portfolio owner".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}
