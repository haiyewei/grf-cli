use crate::cli::args::ListArgs;
use crate::domain::error::Result;
use crate::infra::{paths, store::StateStore};
use crate::ui::output::{print_json, print_loading_table, print_repo_table};

pub fn run(args: ListArgs) -> Result<()> {
    let store = StateStore::new(crate::infra::paths::GrfPaths::discover()?);

    if args.load {
        let cwd = paths::current_dir_utf8()?;
        let loading = store.load_loading()?;
        let entries = store
            .loading_entries_for_workspace(&loading, &cwd)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        if args.json {
            return print_json(&entries);
        }

        if entries.is_empty() {
            println!("No loaded references found in the current workspace.");
        } else {
            print_loading_table(&entries, args.verbose);
        }
        return Ok(());
    }

    let config = store.load_config()?;
    let repos = config.repos.values().cloned().collect::<Vec<_>>();

    if args.json {
        return print_json(&repos);
    }

    if repos.is_empty() {
        println!("No cached repositories found.");
    } else {
        print_repo_table(&repos, args.verbose);
    }

    Ok(())
}
