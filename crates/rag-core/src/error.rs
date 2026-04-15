use thiserror::Error;

#[derive(Error, Debug)]
pub enum RagError {
    #[error("database error: {0}")]
    Database(String),

    #[error("chunk source error: {0}")]
    ChunkSource(String),

    #[error("embedder error: {0}")]
    Embedder(String),

    #[error("assembler error: {0}")]
    Assembler(String),

    #[error("rag error: {0}")]
    Other(String),
}
