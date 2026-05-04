//! Portfolio domain data chunker.
//!
//! Accepts JSON-serialized portfolio entities (jobs, competencies, about
//! sections, social links) and emits one readable-prose [`Chunk`] per entity.
//! Oversize entities are split with the standard sliding-window strategy.

use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let entities: Vec<serde_json::Value> = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut chunks = Vec::new();
    let mut ord = 0usize;

    for entity in &entities {
        let (text, meta) = entity_to_prose(entity);
        if text.is_empty() {
            continue;
        }
        emit_chunks(path, &text, meta, &mut ord, &mut chunks);
    }

    chunks
}

fn entity_to_prose(entity: &serde_json::Value) -> (String, serde_json::Value) {
    if entity.get("company").is_some() && entity.get("title").is_some() {
        return job_to_prose(entity);
    }
    if entity.get("name").is_some() && entity.get("icon").is_some() {
        return competency_to_prose(entity);
    }
    if entity.get("heading").is_some() && entity.get("body").is_some() {
        return about_to_prose(entity);
    }
    if entity.get("platform").is_some() && entity.get("url").is_some() {
        return social_to_prose(entity);
    }
    (String::new(), serde_json::json!({}))
}

fn job_to_prose(job: &serde_json::Value) -> (String, serde_json::Value) {
    let title = job["title"].as_str().unwrap_or("");
    let company = job["company"].as_str().unwrap_or("");
    let start = job["start_date"].as_str().unwrap_or("?");
    let end = job["end_date"].as_str().unwrap_or("present");
    let summary = job["summary"].as_str().unwrap_or("");
    let tech = job["tech_stack"].as_str().unwrap_or("");
    let slug = job["slug"].as_str().unwrap_or("");

    let mut text = format!("Job: {title} at {company} ({start}–{end})");
    if !tech.is_empty() {
        text.push_str(&format!("\nTech: {tech}"));
    }
    if !summary.is_empty() {
        text.push_str(&format!("\nSummary: {summary}"));
    }

    if let Some(details) = job.get("details").and_then(|d| d.as_array()) {
        if !details.is_empty() {
            text.push_str("\nAccomplishments:");
            for d in details {
                let detail_text = d["text"]
                    .as_str()
                    .or_else(|| d["detail_text"].as_str())
                    .unwrap_or("");
                if !detail_text.is_empty() {
                    text.push_str(&format!("\n- {detail_text}"));
                }
            }
        }
    }

    let meta = serde_json::json!({ "entity_type": "job", "slug": slug });
    (text, meta)
}

fn competency_to_prose(comp: &serde_json::Value) -> (String, serde_json::Value) {
    let name = comp["name"].as_str().unwrap_or("");
    let description = comp["description"].as_str().unwrap_or("");
    let slug = comp["slug"].as_str().unwrap_or("");

    let mut text = format!("Competency: {name}");
    if !description.is_empty() {
        text.push_str(&format!("\n{description}"));
    }

    if let Some(evidence) = comp
        .get("evidence")
        .or_else(|| comp.get("highlights"))
        .and_then(|e| e.as_array())
    {
        if !evidence.is_empty() {
            text.push_str("\nEvidence:");
            for e in evidence {
                let highlight = e["text"]
                    .as_str()
                    .or_else(|| e["highlight_text"].as_str())
                    .unwrap_or("");
                let co = e["company"].as_str().unwrap_or("");
                if !highlight.is_empty() {
                    if co.is_empty() {
                        text.push_str(&format!("\n- {highlight}"));
                    } else {
                        text.push_str(&format!("\n- {highlight} ({co})"));
                    }
                }
            }
        }
    }

    let meta = serde_json::json!({ "entity_type": "competency", "slug": slug });
    (text, meta)
}

fn about_to_prose(about: &serde_json::Value) -> (String, serde_json::Value) {
    let heading = about["heading"].as_str().unwrap_or("");
    let body = about["body"].as_str().unwrap_or("");
    let slug = about["slug"].as_str().unwrap_or("");

    let text = format!("About — {heading}\n{body}");
    let meta = serde_json::json!({ "entity_type": "about", "slug": slug });
    (text, meta)
}

fn social_to_prose(social: &serde_json::Value) -> (String, serde_json::Value) {
    let platform = social["platform"].as_str().unwrap_or("");
    let url = social["url"].as_str().unwrap_or("");

    let text = format!("Social: {platform} — {url}");
    let meta = serde_json::json!({ "entity_type": "social_link", "platform": platform });
    (text, meta)
}

fn emit_chunks(
    path: &str,
    text: &str,
    meta_base: serde_json::Value,
    ord: &mut usize,
    chunks: &mut Vec<Chunk>,
) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }
    if words.len() <= MAX_TOKENS {
        let mut meta = meta_base;
        meta["path"] = serde_json::json!(path);
        chunks.push(Chunk {
            ord: *ord,
            content: text.to_owned(),
            token_count: words.len(),
            meta,
        });
        *ord += 1;
    } else {
        let mut start = 0;
        while start < words.len() {
            let end = (start + MAX_TOKENS).min(words.len());
            let chunk_text = words[start..end].join(" ");
            let token_count = end - start;
            let mut meta = meta_base.clone();
            meta["path"] = serde_json::json!(path);
            meta["window_start"] = serde_json::json!(start);
            chunks.push(Chunk {
                ord: *ord,
                content: chunk_text,
                token_count,
                meta,
            });
            *ord += 1;
            if end == words.len() {
                break;
            }
            start += MAX_TOKENS - OVERLAP_WORDS;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job_fixture() -> String {
        serde_json::json!([{
            "slug": "acme-corp",
            "title": "Senior Engineer",
            "company": "Acme Corp",
            "start_date": "2022-01",
            "end_date": "2024-06",
            "tech_stack": "Rust, AWS, Terraform",
            "summary": "Led platform engineering team",
            "details": [
                { "text": "Designed event-driven architecture on AWS Lambda" },
                { "text": "Reduced deploy time from 15min to 2min" },
                { "text": "Mentored 3 junior engineers" }
            ]
        }])
        .to_string()
    }

    fn competency_fixture() -> String {
        serde_json::json!([{
            "slug": "cloud-infra",
            "name": "Cloud Infrastructure",
            "icon": "cloud",
            "description": "Deep expertise in AWS cloud services",
            "highlights": [
                { "text": "Deployed zero-cost Lambda architecture", "company": "Acme Corp" },
                { "text": "Managed multi-region S3 replication", "company": "Beta Inc" }
            ]
        }])
        .to_string()
    }

    #[test]
    fn job_produces_one_chunk_with_details() {
        let chunks = chunk("portfolio/jobs.json", &job_fixture());
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Senior Engineer at Acme Corp"));
        assert!(chunks[0].content.contains("event-driven architecture"));
        assert!(chunks[0].content.contains("Reduced deploy time"));
        assert!(chunks[0].content.contains("Mentored 3 junior"));
    }

    #[test]
    fn competency_produces_one_chunk_with_evidence() {
        let chunks = chunk("portfolio/competencies.json", &competency_fixture());
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Cloud Infrastructure"));
        assert!(chunks[0].content.contains("zero-cost Lambda"));
        assert!(chunks[0].content.contains("(Acme Corp)"));
    }

    #[test]
    fn empty_array_returns_empty() {
        let chunks = chunk("portfolio/empty.json", "[]");
        assert!(chunks.is_empty());
    }

    #[test]
    fn invalid_json_returns_empty() {
        let chunks = chunk("portfolio/bad.json", "not json");
        assert!(chunks.is_empty());
    }

    #[test]
    fn mixed_entities() {
        let data = serde_json::json!([
            { "heading": "Bio", "body": "I am a software engineer", "slug": "bio" },
            { "platform": "GitHub", "url": "https://github.com/test" }
        ])
        .to_string();
        let chunks = chunk("portfolio/mixed.json", &data);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].content.contains("About — Bio"));
        assert!(chunks[1].content.contains("Social: GitHub"));
    }
}
