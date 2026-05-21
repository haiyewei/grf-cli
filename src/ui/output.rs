use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use serde::Serialize;

use crate::domain::error::{GrfError, Result};
use crate::domain::model::{LoadingEntry, RepoRecord};

pub fn print_json<T>(value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    let rendered = serde_json::to_string_pretty(value).map_err(|source| GrfError::JsonParse {
        path: String::from("<stdout>"),
        source,
    })?;
    println!("{rendered}");
    Ok(())
}

pub fn print_repo_table(repos: &[RepoRecord], verbose: bool) {
    let mut table = base_table();
    if verbose {
        table.set_header(vec!["Name", "Branch", "Commit", "Path", "Updated"]);
        for repo in repos {
            table.add_row(vec![
                Cell::new(&repo.name),
                Cell::new(repo.branch.as_deref().unwrap_or("-")),
                Cell::new(short_commit(&repo.commit_id)),
                Cell::new(repo.path.as_str()),
                Cell::new(repo.updated_at),
            ]);
        }
    } else {
        table.set_header(vec!["Name", "Branch", "Commit"]);
        for repo in repos {
            table.add_row(vec![
                Cell::new(&repo.name),
                Cell::new(repo.branch.as_deref().unwrap_or("-")),
                Cell::new(short_commit(&repo.commit_id)),
            ]);
        }
    }
    println!("{table}");
}

pub fn print_loading_table(entries: &[LoadingEntry], verbose: bool) {
    let mut table = base_table();
    if verbose {
        table.set_header(vec!["Repo", "Target", "Branch", "Commit", "Loaded At"]);
        for entry in entries {
            table.add_row(vec![
                Cell::new(&entry.repo_name),
                Cell::new(&entry.target_path),
                Cell::new(entry.branch.as_deref().unwrap_or("-")),
                Cell::new(short_commit(&entry.commit_id)),
                Cell::new(entry.loaded_at),
            ]);
        }
    } else {
        table.set_header(vec!["Repo", "Target", "Commit"]);
        for entry in entries {
            table.add_row(vec![
                Cell::new(&entry.repo_name),
                Cell::new(&entry.target_path),
                Cell::new(short_commit(&entry.commit_id)),
            ]);
        }
    }
    println!("{table}");
}

pub fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn base_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}
