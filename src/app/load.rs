use camino::Utf8Path;
use time::OffsetDateTime;

use crate::app::add::add_repository;
use crate::cli::args::LoadArgs;
use crate::domain::error::{GrfError, Result};
use crate::domain::git_url::looks_like_git_url;
use crate::domain::model::LoadingEntry;
use crate::infra::{
    fs, git, gitignore,
    paths::{self, GrfPaths},
    store::StateStore,
};
use crate::ui::spinner::spinner;

pub fn run(args: LoadArgs) -> Result<()> {
    let paths = GrfPaths::discover()?;
    let store = StateStore::new(paths);
    store.ensure_layout()?;

    let cwd = paths::current_dir_utf8()?;
    let mut config = store.load_config()?;
    let default_shallow = config.defaults.shallow_clone;
    let default_depth = config.defaults.shallow_depth;

    let repo_name = if looks_like_git_url(&args.name) {
        let repo = add_repository(
            &store,
            &mut config,
            &args.name,
            None,
            args.branch.clone(),
            default_shallow,
            Some(default_depth),
        )?;
        repo.name
    } else {
        store.resolve_repo(&config, &args.name)?.name.clone()
    };

    let mut repo = store.resolve_repo(&config, &repo_name)?.clone();
    if let Some(branch) = &args.branch {
        if repo.branch.as_deref() != Some(branch.as_str()) {
            let progress = spinner(&format!("Switching {} to {branch}...", repo.name));
            git::switch_branch(&repo.path, branch)?;
            repo.commit_id = git::current_commit(&repo.path)?;
            repo.branch = git::current_branch(&repo.path)?;
            repo.updated_at = OffsetDateTime::now_utc();
            config.repos.insert(repo.name.clone(), repo.clone());
            store.save_config(&config)?;
            progress.finish_with_message("Branch switched");
        }
    }

    let source_path = determine_source_path(&repo.path, args.subdir.as_deref())?;
    let target_path = determine_target_path(&cwd, &repo.name, args.path.as_deref());
    let target_relative = relative_target_path(&cwd, &target_path)?;

    let progress = spinner("Copying repository...");
    fs::copy_path(
        &source_path,
        &target_path,
        &[
            ".git",
            ".gitreference-meta.json",
            ".grf-meta.json",
            "meta.json",
        ],
    )?;
    progress.finish_with_message("Repository copied");

    if !args.no_ignore {
        gitignore::ensure_entry(&cwd, ".gitreference/")?;
        if !target_relative.starts_with(".gitreference/") {
            gitignore::ensure_entry(&cwd, &dir_gitignore_entry(&target_relative))?;
        }
    }

    let now = OffsetDateTime::now_utc();
    let mut loading = store.load_loading()?;
    let entry = LoadingEntry {
        id: String::new(),
        repo_name: repo.name.clone(),
        repo_url: repo.url.clone(),
        commit_id: repo.commit_id.clone(),
        branch: repo.branch.clone(),
        subdir: args.subdir.clone(),
        target_path: target_relative.clone(),
        loaded_at: now,
        updated_at: Some(now),
        working_directory: cwd.clone(),
    };
    store.upsert_loading_entry(&mut loading, entry);
    store.save_loading(&loading)?;

    println!("Loaded {}", repo.name);
    println!("  Source: {}", source_path);
    println!("  Target: {}", target_path);
    println!("  Commit: {}", short_commit(&repo.commit_id));
    Ok(())
}

fn determine_source_path(
    repo_path: &Utf8Path,
    subdir: Option<&str>,
) -> Result<camino::Utf8PathBuf> {
    let source = match subdir {
        Some(subdir) => repo_path.join(subdir),
        None => repo_path.to_path_buf(),
    };

    if !source.exists() {
        return Err(GrfError::MissingSubdir {
            repo: repo_path.to_string(),
            subdir: subdir.unwrap_or_default().to_string(),
        });
    }

    Ok(source)
}

fn determine_target_path(
    cwd: &Utf8Path,
    repo_name: &str,
    target: Option<&str>,
) -> camino::Utf8PathBuf {
    match target {
        Some(target) => cwd.join(target),
        None => cwd.join(".gitreference").join(repo_name),
    }
}

fn relative_target_path(cwd: &Utf8Path, target: &Utf8Path) -> Result<String> {
    let relative = target
        .strip_prefix(cwd)
        .map_err(|_| GrfError::InvalidArgument {
            message: format!("目标路径不在当前工作区内: {target}"),
        })?;
    Ok(relative.as_str().replace('\\', "/"))
}

fn dir_gitignore_entry(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    }
}

fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}
