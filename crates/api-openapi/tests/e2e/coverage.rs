/// E2e source-scan coverage: no unregistered API structs in `services/ui`.
///
/// Walks `services/ui/src/routes/**/*.rs` and asserts every `pub struct` that
/// derives `Serialize` or `Deserialize` is either:
/// - imported from `api_openapi::models` (present in `ALL_MODELS`), or
/// - listed in `ALLOWED_LOCAL` (templates, internal helpers, etc.).
///
/// This test is in the e2e suite because it crosses the crate boundary and
/// inspects the live `services/ui` source tree.
use std::path::Path;

/// Names defined locally in services/ui that are not API models.
const ALLOWED_LOCAL: &[&str] = &[
    "EmailPayload", // internal Lambda payload, not part of public API
    "RateLimiter",
    "NonceTracker",
    "AppState",
    "AuthConfig",
    "Claims",
    "JwksKey",
    "JwksResponse",
    "SetSessionQuery", // Cognito callback query params
    "AboutQuery",      // query-parameter extractor, not a response model
];

fn is_model_reexport(name: &str) -> bool {
    api_openapi::registry::ALL_MODELS
        .iter()
        .any(|(model_name, _)| *model_name == name)
}

#[test]
fn no_unregistered_api_structs_in_services_ui() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let routes_dir = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("services/ui/src/routes");

    if !routes_dir.exists() {
        eprintln!("SKIP: services/ui/src/routes not found at {:?}", routes_dir);
        return;
    }

    let mut violations: Vec<String> = Vec::new();
    visit_rs_files(&routes_dir, &mut |path, content| {
        for struct_name in extract_serializable_structs(content) {
            if ALLOWED_LOCAL.contains(&struct_name.as_str()) {
                continue;
            }
            if is_model_reexport(&struct_name) {
                continue;
            }
            violations.push(format!("{}: {}", path.display(), struct_name));
        }
    });

    assert!(
        violations.is_empty(),
        "Found structs in services/ui/src/routes not registered in api_openapi::models:\n{}\n\n\
         Add them to crates/api-openapi/src/models/ and register in registry.rs.",
        violations.join("\n")
    );
}

fn visit_rs_files(dir: &Path, cb: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, cb);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                cb(&path, &content);
            }
        }
    }
}

fn extract_serializable_structs(content: &str) -> Vec<String> {
    let has_serde = content.contains("Serialize") || content.contains("Deserialize");
    if !has_serde {
        return Vec::new();
    }
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub struct ") && !trimmed.contains("use ") {
            if let Some(rest) = trimmed.strip_prefix("pub struct ") {
                let name = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('{');
                if !name.is_empty()
                    && name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}
