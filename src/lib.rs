pub mod app;
pub mod cli;
pub mod domain;
pub mod infra;
pub mod ui;

use clap::Parser;
use cli::args::Cli;
use domain::error::Result;

pub fn run() -> Result<()> {
    infra::runtime::init_tracing();
    let cli = Cli::parse();
    cli::run::run(cli)
}
