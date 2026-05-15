//! Resume markdown generation and pandoc conversion

use anyhow::Context;
use llm_anthropic::AnthropicProvider;
use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
use llm_openai::OpenAIProvider;
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::ResumeFormat;

const HEADER: &str = "# Sharful Islam
**AI Systems & Platform Engineer · Rust · RAG · LLM · AWS**

sharfulislam@sislam.com · [GitHub](https://github.com/shantopagla) · [LinkedIn](https://www.linkedin.com/in/sharfulislam/) · [sislam.com](https://sislam.com)

---

";

struct Job {
    id: i64,
    company: String,
    title: String,
    start_date: String,
    end_date: String,
    summary: String,
    tech_stack: Vec<String>,
    resume_display: String,
    sort_order: i64,
}

#[allow(dead_code)]
struct JobDetail {
    job_id: i64,
    detail_text: String,
    category: String,
}

struct Competency {
    id: i64,
    name: String,
    description: String,
}

struct Evidence {
    competency_id: i64,
    job_id: i64,
    text: String,
}

struct SkillCategory {
    name: String,
    skills: Vec<String>,
}

struct Education {
    degree: String,
    institution: String,
    location: Option<String>,
}

pub async fn generate_resume(
    db_path: &Path,
    output_dir: &Path,
    format: &ResumeFormat,
    api_key: Option<String>,
    provider: Option<&str>,
) -> anyhow::Result<()> {
    check_pandoc()?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

    let has_resume_columns = has_column(&conn, "jobs", "resume_display");

    let jobs = load_jobs(&conn, has_resume_columns)?;
    let details = load_job_details(&conn, has_resume_columns)?;
    let competencies = load_competencies(&conn)?;
    let evidence = load_evidence(&conn)?;
    let raw_bio = load_me_bio(&conn)?;
    let curated_skills = load_curated_skills(&conn).unwrap_or_default();
    let education = load_education(&conn).unwrap_or_else(|_| default_education());

    let summary = match (api_key, provider) {
        (Some(key), Some(provider_id)) => {
            println!("  Polishing Professional Summary via {provider_id}...");
            polish_bio_to_summary_ai(&raw_bio, &key, provider_id)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("  Warning: AI polish failed ({e}), using static summary.");
                    polish_bio_to_summary_static(&raw_bio)
                })
        }
        (Some(key), None) => {
            println!("  Polishing Professional Summary via Claude (default)...");
            polish_bio_to_summary_ai(&raw_bio, &key, "anthropic")
                .await
                .unwrap_or_else(|e| {
                    eprintln!("  Warning: AI polish failed ({e}), using static summary.");
                    polish_bio_to_summary_static(&raw_bio)
                })
        }
        (None, _) => polish_bio_to_summary_static(&raw_bio),
    };

    match format {
        ResumeFormat::Chronological => {
            generate_chronological(
                &jobs,
                &details,
                &summary,
                &curated_skills,
                &education,
                output_dir,
            )?;
        }
        ResumeFormat::Functional => {
            generate_functional(
                &jobs,
                &competencies,
                &evidence,
                &summary,
                &curated_skills,
                &education,
                output_dir,
            )?;
        }
        ResumeFormat::All => {
            generate_chronological(
                &jobs,
                &details,
                &summary,
                &curated_skills,
                &education,
                output_dir,
            )?;
            generate_functional(
                &jobs,
                &competencies,
                &evidence,
                &summary,
                &curated_skills,
                &education,
                output_dir,
            )?;
        }
    }

    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return false;
    };
    stmt.query_map([], |row| row.get::<_, String>(1))
        .map(|rows| rows.filter_map(|r| r.ok()).any(|name| name == column))
        .unwrap_or(false)
}

fn default_education() -> Vec<Education> {
    vec![
        Education {
            degree: "B.S. Computing Sciences & Graphic Design".into(),
            institution: "University of Central Oklahoma".into(),
            location: Some("Edmond, OK".into()),
        },
        Education {
            degree: "Certificate, Management & Leadership Skills".into(),
            institution: "NST, Rockhurst University Continuing Education Center".into(),
            location: None,
        },
    ]
}

fn check_pandoc() -> anyhow::Result<()> {
    let status = Command::new("pandoc").arg("--version").output();
    match status {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(anyhow::anyhow!(
            "pandoc not found. Install it with: brew install pandoc\n\
             For PDF support also install: brew install weasyprint"
        )),
    }
}

fn load_jobs(conn: &Connection, has_resume_columns: bool) -> anyhow::Result<Vec<Job>> {
    let sql = if has_resume_columns {
        "SELECT id, company, title, start_date, \
         COALESCE(end_date, 'Present') as end_date, summary, \
         COALESCE(tech_stack, '') as tech_stack, \
         COALESCE(resume_display, 'full') as resume_display, \
         sort_order \
         FROM jobs ORDER BY sort_order"
    } else {
        "SELECT id, company, title, start_date, \
         COALESCE(end_date, 'Present') as end_date, summary, \
         COALESCE(tech_stack, '') as tech_stack, \
         'full' as resume_display, \
         sort_order \
         FROM jobs ORDER BY sort_order"
    };
    let mut stmt = conn.prepare(sql)?;

    let jobs = stmt
        .query_map([], |row| {
            let tech_raw: String = row.get(6)?;
            let tech_stack = tech_raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok(Job {
                id: row.get(0)?,
                company: row.get(1)?,
                title: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                summary: row.get(5)?,
                tech_stack,
                resume_display: row.get(7)?,
                sort_order: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(jobs)
}

fn load_job_details(conn: &Connection, has_resume_columns: bool) -> anyhow::Result<Vec<JobDetail>> {
    let sql = if has_resume_columns {
        "SELECT job_id, detail_text, COALESCE(category, 'responsibility') as category \
         FROM job_details \
         WHERE COALESCE(resume_visible, 1) = 1 \
         ORDER BY job_id, sort_order"
    } else {
        "SELECT job_id, detail_text, COALESCE(category, 'responsibility') as category \
         FROM job_details \
         ORDER BY job_id, sort_order"
    };
    let mut stmt = conn.prepare(sql)?;

    let details = stmt
        .query_map([], |row| {
            Ok(JobDetail {
                job_id: row.get(0)?,
                detail_text: row.get(1)?,
                category: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(details)
}

fn load_competencies(conn: &Connection) -> anyhow::Result<Vec<Competency>> {
    let mut stmt =
        conn.prepare("SELECT id, name, description FROM competencies ORDER BY sort_order")?;

    let comps = stmt
        .query_map([], |row| {
            Ok(Competency {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(comps)
}

fn load_evidence(conn: &Connection) -> anyhow::Result<Vec<Evidence>> {
    let mut stmt = conn.prepare(
        "SELECT ce.competency_id, ce.job_id, \
         COALESCE(ce.highlight_text, jd.detail_text, '') as text \
         FROM competency_evidence ce \
         LEFT JOIN job_details jd ON ce.detail_id = jd.id \
         ORDER BY ce.competency_id, ce.sort_order",
    )?;

    let evidence = stmt
        .query_map([], |row| {
            Ok(Evidence {
                competency_id: row.get(0)?,
                job_id: row.get(1)?,
                text: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(evidence)
}

fn load_me_bio(conn: &Connection) -> anyhow::Result<String> {
    conn.query_row(
        "SELECT body FROM about_sections WHERE slug = 'me-bio'",
        [],
        |row| row.get(0),
    )
    .context("about_sections row with slug='me-bio' not found — DB is missing required data")
}

fn load_curated_skills(conn: &Connection) -> anyhow::Result<Vec<SkillCategory>> {
    let mut stmt = conn.prepare(
        "SELECT sc.name, cs.skill_name \
         FROM curated_skills cs \
         JOIN skill_categories sc ON cs.category_id = sc.id \
         ORDER BY sc.sort_order, cs.sort_order",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut categories: Vec<SkillCategory> = Vec::new();
    for (cat_name, skill) in rows {
        if let Some(last) = categories.last_mut() {
            if last.name == cat_name {
                last.skills.push(skill);
                continue;
            }
        }
        categories.push(SkillCategory {
            name: cat_name,
            skills: vec![skill],
        });
    }

    Ok(categories)
}

fn load_education(conn: &Connection) -> anyhow::Result<Vec<Education>> {
    let mut stmt =
        conn.prepare("SELECT degree, institution, location FROM education ORDER BY sort_order")?;

    let education = stmt
        .query_map([], |row| {
            Ok(Education {
                degree: row.get(0)?,
                institution: row.get(1)?,
                location: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(education)
}

/// Static fallback summary used when `--ai` is absent or the API call fails.
/// Uses the raw_bio sourced from the DB (about_sections.me-bio) directly.
fn polish_bio_to_summary_static(raw_bio: &str) -> String {
    format!("## Professional Summary\n\n{}\n\n", raw_bio)
}

/// Calls LLM to rephrase `raw_bio` into a polished resume Professional Summary section.
///
/// Enforces the grounding contract: the model may only rephrase content present in `raw_bio`.
/// On error, the caller falls back to [`polish_bio_to_summary_static`].
async fn polish_bio_to_summary_ai(
    raw_bio: &str,
    api_key: &str,
    provider_id: &str,
) -> anyhow::Result<String> {
    let provider: Box<dyn LlmProvider> = match provider_id {
        "anthropic" => Box::new(AnthropicProvider::new(api_key)),
        "openai" => Box::new(OpenAIProvider::new(api_key)),
        _ => return Err(anyhow::anyhow!("Unknown provider: {provider_id}")),
    };

    let system = "You are a professional resume writer. \
        Your task is to rephrase the bio provided by the user into a polished, \
        third-person Professional Summary suitable for a technical resume. \
        Output ONLY the summary paragraph(s) — no headers, no preamble, no explanation. \
        You MUST NOT invent credentials, companies, titles, or skills not present in the bio. \
        Keep it to 3–5 sentences."
        .to_owned();

    let user_content =
        format!("Rephrase the following bio into a resume Professional Summary:\n\n{raw_bio}");

    let req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, user_content)],
        system: Some(system),
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: 300,
            temperature: 0.3,
            prompt_version: "polish-bio-v1",
        },
    };

    let resp = provider
        .generate(req)
        .await
        .map_err(|e| anyhow::anyhow!("LLM generate failed: {e}"))?;

    let polished = resp.content.trim().to_owned();
    Ok(format!("## Professional Summary\n\n{polished}\n\n"))
}

fn bullet_budget(sort_order: i64, display: &str) -> usize {
    match display {
        "hidden" => 0,
        "condensed" => 1,
        _ => match sort_order {
            0..=1 => 4,
            _ => 2,
        },
    }
}

fn generate_chronological(
    jobs: &[Job],
    details: &[JobDetail],
    summary: &str,
    curated_skills: &[SkillCategory],
    education: &[Education],
    output_dir: &Path,
) -> anyhow::Result<()> {
    println!("  Generating chronological resume...");

    let mut md = String::new();
    md.push_str(HEADER);
    md.push_str(summary);
    md.push_str("## Experience\n\n");

    let mut details_by_job: HashMap<i64, Vec<&JobDetail>> = HashMap::new();
    for d in details {
        details_by_job.entry(d.job_id).or_default().push(d);
    }

    for job in jobs {
        let budget = bullet_budget(job.sort_order, &job.resume_display);
        if budget == 0 {
            continue;
        }

        if job.resume_display == "condensed" {
            md.push_str(&format!(
                "**{}** — {} ({} – {})\n\n",
                job.title, job.company, job.start_date, job.end_date
            ));
            if let Some(job_details) = details_by_job.get(&job.id) {
                for item in job_details.iter().take(budget) {
                    md.push_str(&format!("- {}\n", item.detail_text));
                }
                md.push('\n');
            }
            continue;
        }

        md.push_str(&format!(
            "### {} — {}\n*{} – {}*\n\n{}\n\n",
            job.title, job.company, job.start_date, job.end_date, job.summary
        ));

        if let Some(job_details) = details_by_job.get(&job.id) {
            for item in job_details.iter().take(budget) {
                md.push_str(&format!("- {}\n", item.detail_text));
            }
            md.push('\n');
        }

        if !job.tech_stack.is_empty() {
            md.push_str(&format!(
                "*Technologies: {}*\n\n",
                job.tech_stack.join(", ")
            ));
        }

        md.push_str("---\n\n");
    }

    render_skills(&mut md, curated_skills, jobs);
    render_education(&mut md, education);

    write_and_convert(&md, output_dir, "chronological")
}

fn generate_functional(
    jobs: &[Job],
    competencies: &[Competency],
    evidence: &[Evidence],
    summary: &str,
    curated_skills: &[SkillCategory],
    education: &[Education],
    output_dir: &Path,
) -> anyhow::Result<()> {
    println!("  Generating functional resume...");

    let mut md = String::new();
    md.push_str(HEADER);
    md.push_str(summary);

    let job_map: HashMap<i64, &Job> = jobs.iter().map(|j| (j.id, j)).collect();

    let mut evidence_by_comp: HashMap<i64, Vec<&Evidence>> = HashMap::new();
    for ev in evidence {
        evidence_by_comp
            .entry(ev.competency_id)
            .or_default()
            .push(ev);
    }

    md.push_str("## Core Competencies\n\n");

    for comp in competencies {
        let has_evidence = evidence_by_comp
            .get(&comp.id)
            .is_some_and(|evs| evs.iter().any(|e| !e.text.is_empty()));
        if !has_evidence {
            continue;
        }

        md.push_str(&format!("### {}\n\n{}\n\n", comp.name, comp.description));

        if let Some(evs) = evidence_by_comp.get(&comp.id) {
            let mut by_company: HashMap<i64, Vec<&&Evidence>> = HashMap::new();
            for ev in evs {
                by_company.entry(ev.job_id).or_default().push(ev);
            }

            for (job_id, items) in &by_company {
                if let Some(job) = job_map.get(job_id) {
                    md.push_str(&format!("*{}*\n\n", job.company));
                    for item in items {
                        if !item.text.is_empty() {
                            md.push_str(&format!("- {}\n", item.text));
                        }
                    }
                    md.push('\n');
                }
            }
        }

        md.push_str("---\n\n");
    }

    md.push_str("## Employment History\n\n");
    for job in jobs {
        if job.resume_display == "hidden" {
            continue;
        }
        md.push_str(&format!(
            "- **{}** @ {} ({} – {})\n",
            job.title, job.company, job.start_date, job.end_date
        ));
    }
    md.push('\n');

    render_skills(&mut md, curated_skills, jobs);
    render_education(&mut md, education);

    write_and_convert(&md, output_dir, "functional")
}

fn render_skills(md: &mut String, curated_skills: &[SkillCategory], jobs: &[Job]) {
    md.push_str("## Technical Skills\n\n");
    if curated_skills.is_empty() {
        let mut seen = std::collections::HashSet::new();
        let all: Vec<&str> = jobs
            .iter()
            .flat_map(|j| j.tech_stack.iter())
            .filter(|t| seen.insert(t.as_str()))
            .map(|t| t.as_str())
            .collect();
        md.push_str(&all.join(", "));
        md.push_str("\n\n");
    } else {
        for cat in curated_skills {
            md.push_str(&format!("**{}:** {}\n\n", cat.name, cat.skills.join(", ")));
        }
    }
}

fn render_education(md: &mut String, education: &[Education]) {
    md.push_str("## Education\n\n");
    for ed in education {
        if let Some(loc) = &ed.location {
            md.push_str(&format!(
                "- **{}** — {}, {}\n",
                ed.degree, ed.institution, loc
            ));
        } else {
            md.push_str(&format!("- **{}** — {}\n", ed.degree, ed.institution));
        }
    }
    md.push('\n');
}

fn write_and_convert(md: &str, output_dir: &Path, format_name: &str) -> anyhow::Result<()> {
    let stem = format!("sharful-islam-resume-{}", format_name);

    let md_path = output_dir.join(format!("{}.md", stem));
    let docx_path = output_dir.join(format!("{}.docx", stem));
    let pdf_path = output_dir.join(format!("{}.pdf", stem));

    // Write markdown
    fs::write(&md_path, md).with_context(|| format!("Failed to write {}", md_path.display()))?;
    println!("  Written: {}", md_path.display());

    // Convert to DOCX
    println!("  Converting to DOCX...");
    run_pandoc(&[md_path.to_str().unwrap(), "-o", docx_path.to_str().unwrap()])
        .with_context(|| "pandoc DOCX conversion failed")?;
    println!("  Written: {}", docx_path.display());

    // Convert to PDF via weasyprint
    println!("  Converting to PDF...");
    run_pandoc(&[
        md_path.to_str().unwrap(),
        "--pdf-engine=weasyprint",
        "-o",
        pdf_path.to_str().unwrap(),
    ])
    .with_context(|| "pandoc PDF conversion failed")?;
    println!("  Written: {}", pdf_path.display());

    Ok(())
}

fn run_pandoc(args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("pandoc")
        .args(args)
        .status()
        .context("Failed to spawn pandoc")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "pandoc exited with status: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
