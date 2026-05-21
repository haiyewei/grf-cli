use camino::Utf8Path;

use crate::domain::error::{GrfError, Result};

const COMMENT: &str = "# Added by grf";

pub fn ensure_entry(dir: &Utf8Path, entry: &str) -> Result<()> {
    let path = dir.join(".gitignore");
    let existing = if path.exists() {
        fs_err::read_to_string(&path)
            .map_err(|source| GrfError::io_read(path.to_string(), source))?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = existing.lines().map(ToOwned::to_owned).collect();
    let normalized = entry.trim().to_string();
    if lines.iter().any(|line| line.trim() == normalized) {
        return Ok(());
    }

    let comment_index = lines.iter().position(|line| line.trim() == COMMENT);
    match comment_index {
        Some(index) => lines.insert(index + 1, normalized),
        None => {
            if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
                lines.push(String::new());
            }
            lines.push(String::from(COMMENT));
            lines.push(normalized);
        }
    }

    fs_err::write(&path, format!("{}\n", lines.join("\n")))
        .map_err(|source| GrfError::io_write(path.to_string(), source))
}

pub fn remove_entry(dir: &Utf8Path, entry: &str) -> Result<bool> {
    let path = dir.join(".gitignore");
    if !path.exists() {
        return Ok(false);
    }

    let existing = fs_err::read_to_string(&path)
        .map_err(|source| GrfError::io_read(path.to_string(), source))?;
    let normalized = entry.trim();
    let mut removed = false;
    let mut new_lines = Vec::new();

    for line in existing.lines() {
        if line.trim() == normalized {
            removed = true;
            continue;
        }
        new_lines.push(line.to_string());
    }

    while new_lines.first().is_some_and(|line| line.trim().is_empty()) {
        new_lines.remove(0);
    }
    while new_lines.last().is_some_and(|line| line.trim().is_empty()) {
        new_lines.pop();
    }

    if removed {
        fs_err::write(
            &path,
            if new_lines.is_empty() {
                String::new()
            } else {
                format!("{}\n", new_lines.join("\n"))
            },
        )
        .map_err(|source| GrfError::io_write(path.to_string(), source))?;
    }

    Ok(removed)
}
