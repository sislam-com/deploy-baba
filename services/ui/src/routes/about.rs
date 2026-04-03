use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use std::sync::Arc;

use crate::db::{load_social_links, Db, SocialLink};

pub struct AboutSection {
    pub heading: String,
    pub body: String,
}

fn query_sections(conn: &rusqlite::Connection, page: &str) -> Vec<AboutSection> {
    let mut stmt = conn
        .prepare(
            "SELECT heading, body FROM about_sections
             WHERE page = ?1 ORDER BY sort_order ASC",
        )
        .expect("prepare about_sections");

    stmt.query_map(rusqlite::params![page], |row| {
        Ok(AboutSection {
            heading: row.get(0)?,
            body: row.get(1)?,
        })
    })
    .expect("query about_sections")
    .filter_map(|r| r.ok())
    .collect()
}

#[derive(Template)]
#[template(path = "about_me.html")]
struct AboutMeTemplate {
    sections: Vec<AboutSection>,
    social_links: Vec<SocialLink>,
}

#[derive(Template)]
#[template(path = "about_repo.html")]
struct AboutRepoTemplate {
    sections: Vec<AboutSection>,
    social_links: Vec<SocialLink>,
}

pub async fn about_me(State(db): State<Arc<Db>>) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let sections = query_sections(&conn, "me");
    let social_links = load_social_links(&conn);
    AboutMeTemplate {
        sections,
        social_links,
    }
}

pub async fn about_repo(State(db): State<Arc<Db>>) -> impl IntoResponse {
    let conn = db.conn.lock().unwrap();
    let sections = query_sections(&conn, "repo");
    let social_links = load_social_links(&conn);
    AboutRepoTemplate {
        sections,
        social_links,
    }
}
