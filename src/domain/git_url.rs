use std::path::Path;

use url::Url;

use crate::domain::error::{GrfError, Result};

#[derive(Debug, Clone)]
pub struct ParsedGitUrl {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub port: Option<u16>,
}

impl ParsedGitUrl {
    pub fn canonical_name(&self) -> String {
        format!("{}/{}/{}", self.storage_host(), self.owner, self.repo)
    }

    pub fn storage_host(&self) -> String {
        match self.port {
            Some(port) => format!("{}_{}", self.host, port),
            None => self.host.clone(),
        }
    }
}

pub fn looks_like_git_url(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("git@")
        || value.starts_with("file://")
        || value.starts_with("git://")
        || Path::new(value).exists()
}

pub fn parse_git_url(raw: &str) -> Result<ParsedGitUrl> {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return parse_http_url(raw);
    }

    if raw.starts_with("git://") {
        return parse_http_like_url(raw);
    }

    if raw.starts_with("file://") {
        return parse_file_url(raw);
    }

    if raw.starts_with("git@") {
        return parse_ssh_url(raw);
    }

    if Path::new(raw).exists() {
        return parse_local_path(raw);
    }

    Err(GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })
}

fn parse_http_url(raw: &str) -> Result<ParsedGitUrl> {
    parse_http_like_url(raw)
}

fn parse_http_like_url(raw: &str) -> Result<ParsedGitUrl> {
    let url = Url::parse(raw).map_err(|_| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?;
    let host = url.host_str().ok_or_else(|| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?;
    let segments: Vec<_> = url
        .path_segments()
        .ok_or_else(|| GrfError::InvalidGitUrl {
            url: raw.to_string(),
        })?
        .filter(|segment| !segment.is_empty())
        .collect();

    if segments.len() < 2 {
        return Err(GrfError::InvalidGitUrl {
            url: raw.to_string(),
        });
    }

    let owner = segments[0].to_string();
    let repo = trim_repo_suffix(segments[1]);

    Ok(ParsedGitUrl {
        host: host.to_string(),
        owner,
        repo,
        port: url.port(),
    })
}

fn parse_ssh_url(raw: &str) -> Result<ParsedGitUrl> {
    let without_prefix = raw
        .strip_prefix("git@")
        .ok_or_else(|| GrfError::InvalidGitUrl {
            url: raw.to_string(),
        })?;
    let (host, path) = without_prefix
        .split_once(':')
        .ok_or_else(|| GrfError::InvalidGitUrl {
            url: raw.to_string(),
        })?;
    let segments: Vec<_> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    if segments.len() != 2 {
        return Err(GrfError::InvalidGitUrl {
            url: raw.to_string(),
        });
    }

    Ok(ParsedGitUrl {
        host: host.to_string(),
        owner: segments[0].to_string(),
        repo: trim_repo_suffix(segments[1]),
        port: None,
    })
}

fn parse_file_url(raw: &str) -> Result<ParsedGitUrl> {
    let url = Url::parse(raw).map_err(|_| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?;
    let file_path = url.to_file_path().map_err(|_| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?;
    parse_local_path(file_path.to_str().ok_or_else(|| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?)
}

fn parse_local_path(raw: &str) -> Result<ParsedGitUrl> {
    let canonical = std::fs::canonicalize(raw).map_err(|_| GrfError::InvalidGitUrl {
        url: raw.to_string(),
    })?;
    let canonical = canonical
        .to_str()
        .ok_or_else(|| GrfError::InvalidGitUrl {
            url: raw.to_string(),
        })?
        .replace('\\', "/");
    let canonical = canonical.trim_end_matches('/');
    let repo_name = canonical
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| GrfError::InvalidGitUrl {
            url: raw.to_string(),
        })?;
    let repo = trim_repo_suffix(repo_name);
    let parent = canonical
        .rsplit_once('/')
        .map(|(head, _)| head)
        .unwrap_or("root");

    Ok(ParsedGitUrl {
        host: String::from("local"),
        owner: sanitize_path_label(parent),
        repo,
        port: None,
    })
}

fn trim_repo_suffix(value: &str) -> String {
    value.strip_suffix(".git").unwrap_or(value).to_string()
}

fn sanitize_path_label(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    let mut last_was_underscore = false;

    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() { ch } else { '_' };
        if normalized == '_' {
            if last_was_underscore {
                continue;
            }
            last_was_underscore = true;
        } else {
            last_was_underscore = false;
        }
        sanitized.push(normalized);
    }

    sanitized.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{looks_like_git_url, parse_git_url};

    #[test]
    fn parses_https_url() {
        let parsed = parse_git_url("https://github.com/openai/codex.git").unwrap();
        assert_eq!(parsed.canonical_name(), "github.com/openai/codex");
    }

    #[test]
    fn parses_http_url_with_port() {
        let parsed = parse_git_url("http://127.0.0.1:3000/team/repo.git").unwrap();
        assert_eq!(parsed.canonical_name(), "127.0.0.1_3000/team/repo");
    }

    #[test]
    fn parses_ssh_url() {
        let parsed = parse_git_url("git@github.com:openai/codex.git").unwrap();
        assert_eq!(parsed.canonical_name(), "github.com/openai/codex");
    }

    #[test]
    fn detects_git_url() {
        assert!(looks_like_git_url("https://github.com/openai/codex.git"));
        assert!(looks_like_git_url("git@github.com:openai/codex.git"));
        assert!(!looks_like_git_url("codex"));
    }

    #[test]
    fn parses_local_path() {
        let temp = TempDir::new().unwrap();
        let repo_dir = temp.path().join("sample.git");
        std::fs::create_dir_all(&repo_dir).unwrap();

        let parsed = parse_git_url(repo_dir.to_str().unwrap()).unwrap();
        assert_eq!(parsed.host, "local");
        assert_eq!(parsed.repo, "sample");
    }
}
