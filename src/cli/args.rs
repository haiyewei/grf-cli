use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "grf",
    version,
    about = "Git Reference - 用全局缓存管理参考仓库，并按需加载到当前项目",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Add(AddArgs),
    Clean(CleanArgs),
    List(ListArgs),
    Load(LoadArgs),
    Unload(UnloadArgs),
    Update(UpdateArgs),
}

#[derive(Debug, Clone, Args)]
pub struct AddArgs {
    pub url: String,

    #[arg(short, long)]
    pub name: Option<String>,

    #[arg(short, long)]
    pub branch: Option<String>,

    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_shallow")]
    pub shallow: bool,

    #[arg(long = "no-shallow", action = ArgAction::SetTrue)]
    pub no_shallow: bool,

    #[arg(long)]
    pub depth: Option<u32>,
}

#[derive(Debug, Clone, Args)]
pub struct CleanArgs {
    pub name: Option<String>,

    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub load: bool,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Args)]
pub struct LoadArgs {
    pub name: String,
    pub path: Option<String>,

    #[arg(short, long)]
    pub subdir: Option<String>,

    #[arg(long = "no-ignore", action = ArgAction::SetTrue)]
    pub no_ignore: bool,

    #[arg(short, long)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UnloadArgs {
    pub name: Option<String>,

    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long)]
    pub list: bool,

    #[arg(long)]
    pub keep_empty: bool,

    #[arg(long = "clean-empty")]
    pub clean_empty: bool,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    pub name: Option<String>,

    #[arg(long)]
    pub check: bool,

    #[arg(long)]
    pub status: bool,

    #[arg(short, long)]
    pub sync: bool,

    #[arg(long = "sync-only")]
    pub sync_only: bool,

    #[arg(short, long)]
    pub force: bool,
}
