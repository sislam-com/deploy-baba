use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize(root: Option<PathBuf>) -> PathBuf {
    let resolved = match root {
        Some(path) => resolve_existing_or_absolute(&path),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let _ = WORKSPACE_ROOT.set(resolved.clone());
    resolved
}

pub fn root() -> PathBuf {
    WORKSPACE_ROOT
        .get()
        .cloned()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root().join(path)
    }
}

fn resolve_existing_or_absolute(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_path_stays_absolute() {
        let path = PathBuf::from("/tmp/example");
        assert_eq!(resolve_existing_or_absolute(&path), path);
    }
}
