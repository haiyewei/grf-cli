use camino::{Utf8Path, Utf8PathBuf};
use dialoguer::Confirm;
use walkdir::WalkDir;

use crate::cli::args::UnloadArgs;
use crate::domain::error::{GrfError, Result};
use crate::domain::model::LoadingEntry;
use crate::infra::{
    fs, gitignore,
    paths::{self, GrfPaths},
    store::StateStore,
};
use crate::ui::output::{print_json, print_lines, print_loading_table};

pub fn run(args: UnloadArgs) -> Result<()> {
    let store = StateStore::new(GrfPaths::discover()?);
    let cwd = paths::current_dir_utf8()?;
    let mut loading = store.load_loading()?;
    let workspace_entries = store
        .loading_entries_for_workspace(&loading, &cwd)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if args.list {
        if workspace_entries.is_empty() {
            println!("No loaded references found in the current workspace.");
        } else {
            print_loading_table(&workspace_entries, args.verbose);
        }
        return Ok(());
    }

    if args.clean_empty {
        let cleaned = prune_empty_dirs(&GrfPaths::workspace_root(&cwd), args.verbose)?;
        println!("Removed {cleaned} empty directories.");
        return Ok(());
    }

    if workspace_entries.is_empty() {
        return Err(GrfError::NoLoadedReferences);
    }

    let selected = if args.all {
        workspace_entries
    } else {
        let Some(name) = args.name.as_deref() else {
            return Err(GrfError::InvalidArgument {
                message: String::from("unload 需要指定仓库名称，或使用 --all / --list。"),
            });
        };
        match_loaded_entries(&workspace_entries, name)?
    };

    if !args.force
        && !Confirm::new()
            .with_prompt(if args.all {
                String::from("Remove all loaded references from the current workspace?")
            } else {
                format!("Remove {} loaded reference(s)?", selected.len())
            })
            .default(false)
            .interact()
            .map_err(|source| GrfError::io_read("<stdin>", source.into()))?
    {
        return Err(GrfError::Cancelled);
    }

    let mut logs = Vec::new();
    for entry in &selected {
        let absolute_target = cwd.join(&entry.target_path);

        if absolute_target.exists() {
            fs::remove_path(&absolute_target)?;
        }

        store.remove_loading_entry(&mut loading, &cwd, &entry.target_path);
        cleanup_gitignore(&cwd, &entry.target_path)?;
        logs.push(format!("Removed {}", absolute_target));

        if !args.keep_empty {
            remove_empty_parents(
                &absolute_target,
                &GrfPaths::workspace_root(&cwd),
                args.verbose,
            )?;
        }
    }

    store.save_loading(&loading)?;
    let root = GrfPaths::workspace_root(&cwd);
    if !root.exists() {
        let _ = gitignore::remove_entry(&cwd, ".gitreference/")?;
    } else if is_dir_empty(&root)? && !args.keep_empty {
        fs::remove_path(&root)?;
        let _ = gitignore::remove_entry(&cwd, ".gitreference/")?;
    }

    print_lines(&logs);
    Ok(())
}

fn match_loaded_entries(entries: &[LoadingEntry], needle: &str) -> Result<Vec<LoadingEntry>> {
    let normalized = needle.replace('\\', "/");
    let matches = entries
        .iter()
        .filter(|entry| {
            entry.repo_name == normalized
                || entry.target_path == normalized
                || entry.repo_name.ends_with(&format!("/{normalized}"))
                || entry.target_path.ends_with(&format!("/{normalized}"))
                || entry
                    .repo_name
                    .split('/')
                    .next_back()
                    .is_some_and(|part| part == normalized)
        })
        .cloned()
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(GrfError::RepoNotFound {
            name: needle.to_string(),
        });
    }

    Ok(matches)
}

fn cleanup_gitignore(cwd: &Utf8Path, target_path: &str) -> Result<()> {
    if target_path.starts_with(".gitreference/") {
        return Ok(());
    }

    let _ = gitignore::remove_entry(cwd, &dir_gitignore_entry(target_path))?;
    Ok(())
}

fn dir_gitignore_entry(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    }
}

fn remove_empty_parents(target: &Utf8Path, stop_at: &Utf8Path, verbose: bool) -> Result<()> {
    let mut current = target
        .parent()
        .map(Utf8PathBuf::from)
        .unwrap_or_else(|| stop_at.to_path_buf());

    while current.starts_with(stop_at) && current != stop_at {
        if !current.exists() || !is_dir_empty(&current)? {
            break;
        }

        if verbose {
            println!("Cleaning empty directory {}", current);
        }
        fs::remove_path(&current)?;
        current = current
            .parent()
            .map(Utf8PathBuf::from)
            .unwrap_or_else(|| stop_at.to_path_buf());
    }

    Ok(())
}

fn prune_empty_dirs(root: &Utf8Path, verbose: bool) -> Result<usize> {
    if !root.exists() {
        return Ok(0);
    }

    let mut cleaned = 0usize;
    for entry in WalkDir::new(root).contents_first(true).min_depth(1) {
        let entry = entry.map_err(|source| GrfError::io_read(root.to_string(), source.into()))?;
        if !entry.file_type().is_dir() {
            continue;
        }

        let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).map_err(|pathbuf| {
            GrfError::NonUtf8Path {
                path: pathbuf.to_string_lossy().to_string(),
            }
        })?;

        if is_dir_empty(&path)? {
            if verbose {
                println!("Removing {}", path);
            }
            fs::remove_path(&path)?;
            cleaned += 1;
        }
    }

    Ok(cleaned)
}

fn is_dir_empty(path: &Utf8Path) -> Result<bool> {
    Ok(fs::read_dir(path)?.is_empty())
}

#[allow(dead_code)]
fn _print_json_entries(entries: &[LoadingEntry]) -> Result<()> {
    print_json(entries)
}
