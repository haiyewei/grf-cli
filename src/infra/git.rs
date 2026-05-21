use std::process::Command;

use camino::Utf8Path;
use which::which;

use crate::domain::error::{GrfError, Result};

#[derive(Debug, Clone, Default)]
pub struct CloneOptions {
    pub branch: Option<String>,
    pub shallow: bool,
    pub depth: Option<u32>,
}

pub fn ensure_git_available() -> Result<()> {
    which("git").map(|_| ()).map_err(|_| GrfError::GitNotFound)
}

pub fn clone_repo(url: &str, dest: &Utf8Path, options: &CloneOptions) -> Result<()> {
    let mut args = vec![String::from("clone")];
    if options.shallow {
        args.push(String::from("--depth"));
        args.push(options.depth.unwrap_or(1).to_string());
    }
    if let Some(branch) = &options.branch {
        args.push(String::from("--branch"));
        args.push(branch.clone());
    }
    args.push(url.to_string());
    args.push(dest.to_string());
    run_git(&args, None).map(|_| ())
}

pub fn fetch(repo: &Utf8Path) -> Result<()> {
    run_git(&[String::from("fetch")], Some(repo)).map(|_| ())
}

pub fn pull(repo: &Utf8Path) -> Result<()> {
    run_git(&[String::from("pull")], Some(repo)).map(|_| ())
}

pub fn current_commit(repo: &Utf8Path) -> Result<String> {
    run_git(
        &[String::from("rev-parse"), String::from("HEAD")],
        Some(repo),
    )
}

pub fn current_branch(repo: &Utf8Path) -> Result<Option<String>> {
    let branch = run_git(
        &[
            String::from("rev-parse"),
            String::from("--abbrev-ref"),
            String::from("HEAD"),
        ],
        Some(repo),
    )?;

    if branch == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(branch))
    }
}

pub fn has_updates(repo: &Utf8Path) -> Result<bool> {
    fetch(repo)?;
    let branch = current_branch(repo)?.unwrap_or_else(|| String::from("HEAD"));
    let target = format!("HEAD..origin/{branch}");
    let count = run_git(
        &[String::from("rev-list"), target, String::from("--count")],
        Some(repo),
    )?;
    Ok(count.parse::<u32>().unwrap_or(0) > 0)
}

pub fn switch_branch(repo: &Utf8Path, branch: &str) -> Result<()> {
    run_git(
        &[
            String::from("config"),
            String::from("remote.origin.fetch"),
            String::from("+refs/heads/*:refs/remotes/origin/*"),
        ],
        Some(repo),
    )?;
    run_git(
        &[
            String::from("fetch"),
            String::from("origin"),
            branch.to_string(),
        ],
        Some(repo),
    )?;
    run_git(
        &[
            String::from("checkout"),
            String::from("-B"),
            branch.to_string(),
            format!("origin/{branch}"),
        ],
        Some(repo),
    )?;
    Ok(())
}

fn run_git(args: &[String], cwd: Option<&Utf8Path>) -> Result<String> {
    ensure_git_available()?;

    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = command
        .output()
        .map_err(|source| GrfError::GitCommandFailed {
            command: render_command(args),
            stderr: source.to_string(),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(GrfError::GitCommandFailed {
            command: render_command(args),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

fn render_command(args: &[String]) -> String {
    if args.is_empty() {
        String::from("git")
    } else {
        format!("git {}", args.join(" "))
    }
}
