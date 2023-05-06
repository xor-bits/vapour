use clap::{Args, Parser};
use clap_complete::Shell;

//

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub enum CliArgs {
    /// Translate appid to app name
    AppId(CliAppId),

    /// Get Proton prefix locations for games
    CompatData(CliCompatData),

    /// Generate shell completions
    Completions(CliCompletions),
}

#[derive(Debug, Clone, Copy, Args)]
pub struct CliAppId {
    /// steam_appid
    pub app_id: u32,
}

#[derive(Debug, Clone, Args)]
pub struct CliCompatData {
    /// Make the regex case sensitive, by default it isn't
    #[clap(short, long)]
    pub case_sensitive: bool,

    /// Sort the output by game name
    #[clap(short, long)]
    pub sort: bool,

    /// basically appends "pfx/drive_c" to results
    #[clap(short, long)]
    pub drive_c: bool,

    /// Regex pattern for game names
    pub regex: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct CliCompletions {
    pub shell: Shell,
}
