use service_protocol::ServiceResponse;
use tracing::info;

pub async fn ask_handler(_body: Option<String>) -> ServiceResponse {
    info!("ask handler invoked");

    // TODO: Wire up rag-core retrieval + llm-core generation
    // This is a stub for now — the full implementation requires
    // RagStore initialization and LLM provider setup.

    ServiceResponse::ok(serde_json::json!({
        "answer": "RAG service placeholder — full implementation pending",
        "citations": [],
    }))
}
