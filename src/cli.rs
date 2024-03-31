use clap::{Args, Parser};
use clap_complete::Shell;

//

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub enum CliArgs {
    /// Update appid database
    Update,

    /// Translate appid to app name
    NameOf(CliNameOf),

    /// Get appids of games
    IdOf(CliIdOf),

    /// Get Proton prefix locations of installed games
    CompatData(CliCompatData),

    /// Generate shell completions
    Completions(CliCompletions),
}

#[derive(Debug, Clone, Copy, Args)]
pub struct CliNameOf {
    /// steam_appid
    pub app_id: u32,
}

#[derive(Debug, Clone, Args)]
pub struct CliIdOf {
    /// Make the regex case sensitive, by default it isn't
    #[clap(short, long)]
    pub case_sensitive: bool,

    /// Only include installed games
    #[clap(short, long)]
    pub installed: bool,

    /// Regex pattern for game names
    pub regex: String,
}

#[derive(Debug, Clone, Args)]
pub struct CliCompatData {
    /// Make the regex case sensitive, by default it isn't
    #[clap(short, long)]
    pub case_sensitive: bool,

    /// basically appends "pfx/drive_c" to results
    #[clap(short, long)]
    pub drive_c: bool,

    /// Regex pattern for game names
    pub regex: String,
}

#[derive(Debug, Clone, Args)]
pub struct CliCompletions {
    pub shell: Shell,
}
