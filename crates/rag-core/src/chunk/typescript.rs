use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

static ITEM_KEYWORDS: &[&str] = &[
    "export default function ",
    "export default async function ",
    "export async function ",
    "export function ",
    "export const ",
    "export let ",
    "export interface ",
    "export type ",
    "export enum ",
    "export class ",
    "export abstract class ",
    "function ",
    "async function ",
    "const ",
    "let ",
    "interface ",
    "type ",
    "enum ",
    "class ",
    "abstract class ",
    "describe(",
    "it(",
    "test(",
];

fn is_item_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    ITEM_KEYWORDS.iter().any(|kw| trimmed.starts_with(kw))
}

pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut ord = 0usize;
    let mut current: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut in_item = false;

    for line in content.lines() {
        let open = line.chars().filter(|&c| c == '{' || c == '(').count() as i32;
        let close = line.chars().filter(|&c| c == '}' || c == ')').count() as i32;

        if !in_item {
            if is_item_start(line) {
                in_item = true;
                current.push(line);
                depth = open - close;
                if depth <= 0 && !line.contains('{') && !line.contains('(') {
                    emit_chunks(path, &current.join("\n"), &mut ord, &mut chunks);
                    current.clear();
                    in_item = false;
                    depth = 0;
                }
            }
        } else {
            current.push(line);
            depth += open - close;
            if depth <= 0 {
                emit_chunks(path, &current.join("\n"), &mut ord, &mut chunks);
                current.clear();
                depth = 0;
                in_item = false;
            }
        }
    }

    if !current.is_empty() {
        let text = current.join("\n");
        if !text.trim().is_empty() {
            emit_chunks(path, &text, &mut ord, &mut chunks);
        }
    }

    chunks
}

fn emit_chunks(path: &str, text: &str, ord: &mut usize, chunks: &mut Vec<Chunk>) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }
    if words.len() <= MAX_TOKENS {
        chunks.push(Chunk {
            ord: *ord,
            content: text.to_owned(),
            token_count: words.len(),
            meta: serde_json::json!({ "path": path }),
        });
        *ord += 1;
    } else {
        let mut start = 0;
        while start < words.len() {
            let end = (start + MAX_TOKENS).min(words.len());
            let chunk_text = words[start..end].join(" ");
            let token_count = end - start;
            chunks.push(Chunk {
                ord: *ord,
                content: chunk_text,
                token_count,
                meta: serde_json::json!({ "path": path, "window_start": start }),
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

    const FIXTURE: &str = r#"
import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'

const AUTH_BASE = ''

export default function Login() {
  const navigate = useNavigate()
  const [username, setUsername] = useState('')
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const resp = await fetch(`${AUTH_BASE}/api/auth/signin`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username }),
    })
    if (resp.ok) navigate('/dashboard')
  }

  return (
    <form onSubmit={handleSubmit}>
      <input value={username} onChange={e => setUsername(e.target.value)} />
      {error && <p>{error}</p>}
      <button type="submit">Login</button>
    </form>
  )
}
"#;

    #[test]
    fn produces_non_empty_chunks() {
        let chunks = chunk("web/src/routes/auth/Login.tsx", FIXTURE);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn captures_component_function() {
        let chunks = chunk("web/src/routes/auth/Login.tsx", FIXTURE);
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("Login"), "should contain component name");
        assert!(joined.contains("signin"), "should contain API call");
    }

    #[test]
    fn captures_hook() {
        let hook = r#"
import { useEffect, useState } from 'react'

export function useAuth(redirect = true) {
  const [state, setState] = useState({ loading: true, authenticated: false })

  useEffect(() => {
    fetch('/api/auth/me')
      .then(r => r.json())
      .then(data => setState({ loading: false, authenticated: data.authenticated }))
  }, [])

  return state
}
"#;
        let chunks = chunk("web/src/hooks/useAuth.ts", hook);
        assert!(!chunks.is_empty());
        let joined: String = chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("useAuth"), "should contain hook name");
        assert!(
            joined.contains("/api/auth/me"),
            "should contain API endpoint"
        );
    }

    #[test]
    fn captures_interface() {
        let ts = "export interface AuthState {\n  loading: boolean\n  authenticated: boolean\n  email: string | null\n}\n";
        let chunks = chunk("web/src/types.ts", ts);
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("AuthState"));
    }

    #[test]
    fn empty_file_produces_no_chunks() {
        let chunks = chunk("web/src/empty.ts", "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn chunks_have_ascending_ord() {
        let chunks = chunk("web/src/routes/auth/Login.tsx", FIXTURE);
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
        }
    }
}
