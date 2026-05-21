use time::OffsetDateTime;

use crate::cli::args::UpdateArgs;
use crate::domain::error::{GrfError, Result};
use crate::domain::model::LoadingEntry;
use crate::infra::{
    fs, git,
    paths::{self, GrfPaths},
    store::StateStore,
};
use crate::ui::output::print_lines;

pub fn run(args: UpdateArgs) -> Result<()> {
    let store = StateStore::new(GrfPaths::discover()?);
    let cwd = paths::current_dir_utf8()?;
    let mut config = store.load_config()?;
    let mut loading = store.load_loading()?;

    if args.status {
        let lines = sync_status_lines(&store, &config, &loading, &cwd)?;
        print_lines(&lines);
        return Ok(());
    }

    if args.sync_only {
        let lines = sync_workspace(&store, &config, &mut loading, &cwd, args.force)?;
        store.save_loading(&loading)?;
        print_lines(&lines);
        return Ok(());
    }

    let target_names = if let Some(name) = args.name.as_deref() {
        vec![store.resolve_repo(&config, name)?.name.clone()]
    } else {
        config.repos.keys().cloned().collect::<Vec<_>>()
    };

    if target_names.is_empty() {
        println!("No cached repositories found.");
        return Ok(());
    }

    let mut lines = Vec::new();
    for name in target_names {
        let mut repo = config
            .repos
            .get(&name)
            .cloned()
            .ok_or_else(|| GrfError::RepoNotFound { name: name.clone() })?;

        let has_updates = git::has_updates(&repo.path)?;
        if args.check {
            lines.push(format!(
                "{}: {}",
                repo.name,
                if has_updates {
                    "updates available"
                } else {
                    "up-to-date"
                }
            ));
            continue;
        }

        if has_updates {
            git::pull(&repo.path)?;
            repo.commit_id = git::current_commit(&repo.path)?;
            repo.branch = git::current_branch(&repo.path)?;
            repo.updated_at = OffsetDateTime::now_utc();
            config.repos.insert(repo.name.clone(), repo.clone());
            lines.push(format!(
                "{}: updated to {}",
                repo.name,
                short_commit(&repo.commit_id)
            ));
        } else {
            lines.push(format!("{}: already up-to-date", repo.name));
        }
    }

    if !args.check {
        store.save_config(&config)?;
    }

    if args.sync && !args.check {
        let sync_lines = sync_workspace(&store, &config, &mut loading, &cwd, args.force)?;
        lines.extend(sync_lines);
        store.save_loading(&loading)?;
    }

    print_lines(&lines);
    Ok(())
}

fn sync_status_lines(
    store: &StateStore,
    config: &crate::domain::model::ConfigFile,
    loading: &crate::domain::model::LoadingFile,
    cwd: &camino::Utf8Path,
) -> Result<Vec<String>> {
    let entries = store.loading_entries_for_workspace(loading, cwd);
    if entries.is_empty() {
        return Err(GrfError::NoLoadedReferences);
    }

    let mut lines = Vec::new();
    for entry in entries {
        let repo = config
            .repos
            .get(&entry.repo_name)
            .ok_or_else(|| GrfError::RepoNotFound {
                name: entry.repo_name.clone(),
            })?;
        let cache_commit = git::current_commit(&repo.path)?;
        let target_exists = cwd.join(&entry.target_path).exists();
        let needs_sync = !target_exists || entry.commit_id != cache_commit;
        lines.push(format!(
            "{} -> {} | cache {} | loaded {} | {}",
            entry.repo_name,
            entry.target_path,
            short_commit(&cache_commit),
            short_commit(&entry.commit_id),
            if needs_sync { "needs sync" } else { "in sync" }
        ));
    }
    Ok(lines)
}

fn sync_workspace(
    store: &StateStore,
    config: &crate::domain::model::ConfigFile,
    loading: &mut crate::domain::model::LoadingFile,
    cwd: &camino::Utf8Path,
    force: bool,
) -> Result<Vec<String>> {
    let entries = store
        .loading_entries_for_workspace(loading, cwd)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Ok(vec![String::from("No loaded references need syncing.")]);
    }

    let mut lines = Vec::new();
    for entry in entries {
        let repo = config
            .repos
            .get(&entry.repo_name)
            .ok_or_else(|| GrfError::RepoNotFound {
                name: entry.repo_name.clone(),
            })?;
        let cache_commit = git::current_commit(&repo.path)?;
        let target = cwd.join(&entry.target_path);
        let needs_sync = force || !target.exists() || entry.commit_id != cache_commit;

        if !needs_sync {
            lines.push(format!("{}: already in sync", entry.repo_name));
            continue;
        }

        let source = match &entry.subdir {
            Some(subdir) => repo.path.join(subdir),
            None => repo.path.clone(),
        };
        if !source.exists() {
            return Err(GrfError::MissingSubdir {
                repo: repo.name.clone(),
                subdir: entry.subdir.clone().unwrap_or_default(),
            });
        }

        if target.exists() {
            fs::remove_path(&target)?;
        }
        fs::copy_path(
            &source,
            &target,
            &[
                ".git",
                ".gitreference-meta.json",
                ".grf-meta.json",
                "meta.json",
            ],
        )?;
        refresh_loading_entry(loading, cwd, &entry, &cache_commit);
        lines.push(format!(
            "{}: synced to {}",
            entry.repo_name,
            short_commit(&cache_commit)
        ));
    }

    Ok(lines)
}

fn refresh_loading_entry(
    loading: &mut crate::domain::model::LoadingFile,
    cwd: &camino::Utf8Path,
    entry: &LoadingEntry,
    commit_id: &str,
) {
    if let Some(current) = loading.entries.iter_mut().find(|candidate| {
        candidate.working_directory == cwd && candidate.target_path == entry.target_path
    }) {
        current.commit_id = commit_id.to_string();
        current.updated_at = Some(OffsetDateTime::now_utc());
    }
}

fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}
