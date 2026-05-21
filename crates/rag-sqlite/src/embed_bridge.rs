//! Bridge between `llm-core::EmbeddingProvider` and `rag-core::Embedder`.

use async_trait::async_trait;
use llm_core::EmbeddingProvider;
use rag_core::{Embedder, RagError};
use std::sync::Arc;

/// Adapter that wraps an [`EmbeddingProvider`] (from `llm-core`) and implements
/// the [`Embedder`] trait (from `rag-core`).
pub struct LlmEmbedder {
    inner: Arc<dyn EmbeddingProvider>,
}

impl LlmEmbedder {
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { inner: provider }
    }
}

#[async_trait]
impl Embedder for LlmEmbedder {
    fn provider_id(&self) -> &'static str {
        self.inner.provider_id()
    }

    fn dim(&self) -> usize {
        self.inner.embedding_dim()
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, RagError> {
        let owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        self.inner
            .embed(&owned)
            .await
            .map_err(|e| RagError::Embedder(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_core::LlmError;

    struct StubEmbeddingProvider {
        dim: usize,
    }

    #[async_trait]
    impl EmbeddingProvider for StubEmbeddingProvider {
        fn provider_id(&self) -> &'static str {
            "stub"
        }
        fn embedding_dim(&self) -> usize {
            self.dim
        }
        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError> {
            Ok(texts.iter().map(|_| vec![0.1_f32; self.dim]).collect())
        }
    }

    struct FailingEmbeddingProvider;

    #[async_trait]
    impl EmbeddingProvider for FailingEmbeddingProvider {
        fn provider_id(&self) -> &'static str {
            "failing"
        }
        fn embedding_dim(&self) -> usize {
            1536
        }
        async fn embed(&self, _: &[String]) -> Result<Vec<Vec<f32>>, LlmError> {
            Err(LlmError::Upstream {
                message: "test failure".to_string(),
            })
        }
    }

    #[test]
    fn bridge_provider_id_passthrough() {
        let inner = Arc::new(StubEmbeddingProvider { dim: 1536 });
        let bridge = LlmEmbedder::new(inner);
        assert_eq!(bridge.provider_id(), "stub");
    }

    #[test]
    fn bridge_dim_passthrough() {
        let inner = Arc::new(StubEmbeddingProvider { dim: 768 });
        let bridge = LlmEmbedder::new(inner);
        assert_eq!(bridge.dim(), 768);
    }

    #[tokio::test]
    async fn bridge_converts_str_slice_to_string_vec() {
        let inner = Arc::new(StubEmbeddingProvider { dim: 3 });
        let bridge = LlmEmbedder::new(inner);
        let result = bridge.embed(&["hello", "world"]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 3);
    }

    #[tokio::test]
    async fn bridge_maps_llm_error_to_rag_error() {
        let inner = Arc::new(FailingEmbeddingProvider);
        let bridge = LlmEmbedder::new(inner);
        let err = bridge.embed(&["test"]).await.unwrap_err();
        match err {
            RagError::Embedder(msg) => assert!(msg.contains("test failure")),
            other => panic!("expected Embedder error, got: {other:?}"),
        }
    }
}
