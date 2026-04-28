use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use std::sync::Arc;

use crate::db::{load_social_links, Db, SocialLink};

#[derive(Template)]
#[template(path = "ask.html")]
struct AskTemplate {
    social_links: Vec<SocialLink>,
    examples: Vec<&'static str>,
}

pub async fn ask_page(State(db): State<Arc<Db>>) -> impl IntoResponse {
    let social_links = {
        let conn = db.conn.lock().unwrap();
        load_social_links(&conn)
    };

    AskTemplate {
        social_links,
        examples: vec![
            "Why SQLite instead of PostgreSQL?",
            "How does Lambda load secrets at cold start?",
            "How does the PoW challenge protect the contact form?",
            "What is the RAG pipeline and how does it work?",
            "How is Cognito authentication implemented?",
            "What are the ADRs for infrastructure decisions?",
        ],
    }
}
