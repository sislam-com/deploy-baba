//! Resume markdown generation and pandoc conversion

use anyhow::Context;
use llm_anthropic::AnthropicProvider;
use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::ResumeFormat;

const HEADER: &str = "# Sharful Islam
**AI Systems & Platform Engineer · Rust · RAG · LLM · AWS · 20+ Years**

contact-sislam@sislam.com · [GitHub](https://github.com/shantopagla) · [LinkedIn](https://www.linkedin.com/in/sharfulislam/) · [sislam.com](https://sislam.com)

---

";

const EDUCATION: &str = "## Education

- **B.S. Computing Sciences & Graphic Design** — University of Central Oklahoma, Edmond, OK
- **Certificate, Management & Leadership Skills** — NST, Rockhurst University Continuing Education Center

";

struct Job {
    id: i64,
    company: String,
    title: String,
    start_date: String,
    end_date: String,
    summary: String,
    tech_stack: Vec<String>,
}

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

pub async fn generate_resume(
    db_path: &Path,
    output_dir: &Path,
    format: &ResumeFormat,
    api_key: Option<String>,
) -> anyhow::Result<()> {
    check_pandoc()?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

    let jobs = load_jobs(&conn)?;
    let details = load_job_details(&conn)?;
    let competencies = load_competencies(&conn)?;
    let evidence = load_evidence(&conn)?;
    let raw_bio = load_me_bio(&conn)?;

    let summary = match api_key {
        Some(key) => {
            println!("  Polishing Professional Summary via Claude...");
            polish_bio_to_summary_ai(&raw_bio, &key)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("  Warning: AI polish failed ({e}), using static summary.");
                    polish_bio_to_summary_static(&raw_bio)
                })
        }
        None => polish_bio_to_summary_static(&raw_bio),
    };

    match format {
        ResumeFormat::Chronological => {
            generate_chronological(&jobs, &details, &summary, output_dir)?;
        }
        ResumeFormat::Functional => {
            generate_functional(
                &jobs,
                &details,
                &competencies,
                &evidence,
                &summary,
                output_dir,
            )?;
        }
        ResumeFormat::All => {
            generate_chronological(&jobs, &details, &summary, output_dir)?;
            generate_functional(
                &jobs,
                &details,
                &competencies,
                &evidence,
                &summary,
                output_dir,
            )?;
        }
    }

    Ok(())
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

fn load_jobs(conn: &Connection) -> anyhow::Result<Vec<Job>> {
    let mut stmt = conn.prepare(
        "SELECT id, company, title, start_date, \
         COALESCE(end_date, 'Present') as end_date, summary, \
         COALESCE(tech_stack, '') as tech_stack \
         FROM jobs ORDER BY sort_order",
    )?;

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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(jobs)
}

fn load_job_details(conn: &Connection) -> anyhow::Result<Vec<JobDetail>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, detail_text, COALESCE(category, 'responsibility') as category \
         FROM job_details ORDER BY job_id, sort_order",
    )?;

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

/// Static fallback summary used when `--ai` is absent or the API call fails.
/// Uses the raw_bio sourced from the DB (about_sections.me-bio) directly.
fn polish_bio_to_summary_static(raw_bio: &str) -> String {
    format!("## Professional Summary\n\n{}\n\n", raw_bio)
}

/// Calls Claude to rephrase `raw_bio` into a polished resume Professional Summary section.
///
/// Enforces the grounding contract: the model may only rephrase content present in `raw_bio`.
/// On error, the caller falls back to [`polish_bio_to_summary_static`].
async fn polish_bio_to_summary_ai(raw_bio: &str, api_key: &str) -> anyhow::Result<String> {
    let provider = AnthropicProvider::new(api_key);

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

fn generate_chronological(
    jobs: &[Job],
    details: &[JobDetail],
    summary: &str,
    output_dir: &Path,
) -> anyhow::Result<()> {
    println!("  Generating chronological resume...");

    let mut md = String::new();
    md.push_str(HEADER);
    md.push_str(summary);
    md.push_str("## Experience\n\n");

    // Group details by job_id
    let mut details_by_job: HashMap<i64, Vec<&JobDetail>> = HashMap::new();
    for d in details {
        details_by_job.entry(d.job_id).or_default().push(d);
    }

    for job in jobs {
        md.push_str(&format!(
            "### {} — {}\n*{} – {}*\n\n{}\n\n",
            job.title, job.company, job.start_date, job.end_date, job.summary
        ));

        if let Some(job_details) = details_by_job.get(&job.id) {
            // Group by category
            let mut by_cat: HashMap<&str, Vec<&&JobDetail>> = HashMap::new();
            for d in job_details {
                by_cat.entry(&d.category).or_default().push(d);
            }

            // Category display order
            for cat in &["achievement", "responsibility", "sub-engagement"] {
                if let Some(items) = by_cat.get(*cat) {
                    let label = match *cat {
                        "achievement" => "**Achievements**",
                        "responsibility" => "**Responsibilities**",
                        "sub-engagement" => "**Client Engagements**",
                        _ => cat,
                    };
                    md.push_str(&format!("{}\n\n", label));
                    for item in items {
                        md.push_str(&format!("- {}\n", item.detail_text));
                    }
                    md.push('\n');
                }
            }
        }

        if !job.tech_stack.is_empty() {
            md.push_str(&format!(
                "*Technologies: {}*\n\n",
                job.tech_stack.join(", ")
            ));
        }

        md.push_str("---\n\n");
    }

    // Aggregate tech skills
    let all_tech = aggregate_tech(jobs);
    md.push_str("## Technical Skills\n\n");
    md.push_str(&all_tech);
    md.push('\n');

    md.push_str(EDUCATION);

    write_and_convert(&md, output_dir, "chronological")
}

fn generate_functional(
    jobs: &[Job],
    _details: &[JobDetail],
    competencies: &[Competency],
    evidence: &[Evidence],
    summary: &str,
    output_dir: &Path,
) -> anyhow::Result<()> {
    println!("  Generating functional resume...");

    let mut md = String::new();
    md.push_str(HEADER);
    md.push_str(summary);

    // Build lookup maps
    let job_map: HashMap<i64, &Job> = jobs.iter().map(|j| (j.id, j)).collect();

    // Group evidence by competency
    let mut evidence_by_comp: HashMap<i64, Vec<&Evidence>> = HashMap::new();
    for ev in evidence {
        evidence_by_comp
            .entry(ev.competency_id)
            .or_default()
            .push(ev);
    }

    md.push_str("## Core Competencies\n\n");

    for comp in competencies {
        md.push_str(&format!("### {}\n\n{}\n\n", comp.name, comp.description));

        if let Some(evs) = evidence_by_comp.get(&comp.id) {
            // Group by company
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
        md.push_str(&format!(
            "- **{}** @ {} ({} – {})\n",
            job.title, job.company, job.start_date, job.end_date
        ));
    }
    md.push('\n');

    let all_tech = aggregate_tech(jobs);
    md.push_str("## Technical Skills\n\n");
    md.push_str(&all_tech);
    md.push('\n');

    md.push_str(EDUCATION);

    write_and_convert(&md, output_dir, "functional")
}

fn aggregate_tech(jobs: &[Job]) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut all: Vec<String> = Vec::new();
    for job in jobs {
        for tech in &job.tech_stack {
            if seen.insert(tech.clone()) {
                all.push(tech.clone());
            }
        }
    }
    format!("{}\n", all.join(", "))
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
