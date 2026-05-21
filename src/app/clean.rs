use dialoguer::Confirm;

use crate::cli::args::CleanArgs;
use crate::domain::error::{GrfError, Result};
use crate::infra::{fs, paths::GrfPaths, store::StateStore};

pub fn run(args: CleanArgs) -> Result<()> {
    let store = StateStore::new(GrfPaths::discover()?);
    let mut config = store.load_config()?;

    if config.repos.is_empty() {
        println!("No cached repositories found.");
        return Ok(());
    }

    if args.all {
        if !args.force
            && !Confirm::new()
                .with_prompt("Remove all cached repositories?")
                .default(false)
                .interact()
                .map_err(|source| GrfError::io_read("<stdin>", source.into()))?
        {
            return Err(GrfError::Cancelled);
        }

        let repos = config.repos.values().cloned().collect::<Vec<_>>();
        for repo in repos {
            fs::remove_path(&repo.path)?;
        }
        config.repos.clear();
        store.save_config(&config)?;
        println!("Removed all cached repositories.");
        return Ok(());
    }

    let Some(name) = args.name.as_deref() else {
        return Err(GrfError::InvalidArgument {
            message: String::from("clean 需要指定仓库名称，或使用 --all。"),
        });
    };

    let repo = store.resolve_repo(&config, name)?.clone();
    if !args.force
        && !Confirm::new()
            .with_prompt(format!("Remove cached repository `{}`?", repo.name))
            .default(false)
            .interact()
            .map_err(|source| GrfError::io_read("<stdin>", source.into()))?
    {
        return Err(GrfError::Cancelled);
    }

    fs::remove_path(&repo.path)?;
    config.repos.remove(&repo.name);
    store.save_config(&config)?;
    println!("Removed {}", repo.name);
    Ok(())
}
