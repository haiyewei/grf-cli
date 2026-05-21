use camino::Utf8PathBuf;
use time::OffsetDateTime;

use crate::cli::args::AddArgs;
use crate::domain::error::{GrfError, Result};
use crate::domain::git_url::parse_git_url;
use crate::domain::model::{ConfigFile, RepoRecord};
use crate::infra::{git, paths::GrfPaths, store::StateStore};
use crate::ui::spinner::spinner;

pub fn run(args: AddArgs) -> Result<()> {
    let paths = GrfPaths::discover()?;
    let store = StateStore::new(paths);
    store.ensure_layout()?;

    let mut config = store.load_config()?;
    let repo = add_repository(
        &store,
        &mut config,
        &args.url,
        args.name,
        args.branch,
        !args.no_shallow,
        args.depth,
    )?;

    println!("Added {}", repo.name);
    println!("  URL: {}", repo.url);
    println!("  Path: {}", repo.path);
    println!("  Commit: {}", short_commit(&repo.commit_id));
    Ok(())
}

pub fn add_repository(
    store: &StateStore,
    config: &mut ConfigFile,
    url: &str,
    explicit_name: Option<String>,
    branch: Option<String>,
    shallow: bool,
    depth: Option<u32>,
) -> Result<RepoRecord> {
    let parsed = parse_git_url(url)?;
    let repo_name = explicit_name.unwrap_or_else(|| parsed.canonical_name());
    let repo_path = build_repo_storage_path(store.paths(), &parsed);

    if config.repos.contains_key(&repo_name) {
        return Err(GrfError::RepoAlreadyExists { name: repo_name });
    }

    if let Some(existing) = config
        .repos
        .values()
        .find(|repo| repo.url == url || repo.path == repo_path)
    {
        return Err(GrfError::RepoAlreadyExists {
            name: existing.name.clone(),
        });
    }

    if repo_path.exists() {
        return Err(GrfError::RepoAlreadyExists {
            name: repo_name.clone(),
        });
    }

    git::ensure_git_available()?;

    let progress = spinner("Cloning repository...");
    let clone_options = git::CloneOptions {
        branch: branch.clone(),
        shallow,
        depth,
    };
    git::clone_repo(url, &repo_path, &clone_options)?;
    let commit_id = git::current_commit(&repo_path)?;
    let branch_name = git::current_branch(&repo_path)?;
    progress.finish_with_message("Repository cloned");

    let now = OffsetDateTime::now_utc();
    let repo = RepoRecord {
        name: repo_name.clone(),
        url: url.to_string(),
        path: repo_path,
        added_at: now,
        updated_at: now,
        commit_id,
        branch: branch_name,
    };

    config.repos.insert(repo_name, repo.clone());
    store.save_config(config)?;
    Ok(repo)
}

fn build_repo_storage_path(
    paths: &GrfPaths,
    parsed: &crate::domain::git_url::ParsedGitUrl,
) -> Utf8PathBuf {
    paths
        .repos_root
        .join(parsed.storage_host())
        .join(&parsed.owner)
        .join(&parsed.repo)
}

fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}
