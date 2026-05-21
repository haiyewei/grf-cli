use crate::app;
use crate::cli::args::{Cli, Commands};
use crate::domain::error::Result;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add(args) => app::add::run(args),
        Commands::Clean(args) => app::clean::run(args),
        Commands::List(args) => app::list::run(args),
        Commands::Load(args) => app::load::run(args),
        Commands::Unload(args) => app::unload::run(args),
        Commands::Update(args) => app::update::run(args),
    }
}
