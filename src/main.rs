use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use owo_colors::OwoColorize;
use regex::RegexBuilder;
use std::{cmp::Reverse, collections::BinaryHeap, io::IsTerminal};

use std::{io, os, path::Path};

use self::cli::{CliArgs, CliCompatData, CliCompletions, CliIdOf, CliNameOf};

//

mod cli;
mod db;
mod library;
mod vdf;

//

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: CliArgs = CliArgs::parse();

    let simple = !std::io::stdout().is_terminal();

    match args {
        CliArgs::Update => {
            let appids = db::open_db().await?;
            db::update_appids(&appids).await?;
        }
        CliArgs::NameOf(CliNameOf { app_id }) => {
            let appids = db::open_db().await?;

            let app = appids
                .get(app_id.to_le_bytes())?
                .ok_or_else(|| anyhow!("App not found!"))?;

            let app_name = std::str::from_utf8(app.as_ref())
                .map_err(|err| anyhow!("Invalid database: {err}"))?;

            println!("{app_name}")
        }
        CliArgs::IdOf(CliIdOf {
            case_sensitive,
            installed,
            regex,
        }) => {
            let appids = db::open_db().await?;

            let regex = RegexBuilder::new(regex.as_str())
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|err| anyhow!("App name regex must be valid: {err}"))?;

            let mut sorted = BinaryHeap::new();

            if installed {
                let libs = library::load_steam_libraries()?;

                for app in libs.apps(&appids) {
                    let Some(app_name) = app.app_name() else {
                        continue;
                    };

                    if !regex.is_match(app_name) {
                        continue;
                    }

                    sorted.push((Reverse(app_name.to_string()), app.app_id));
                }
            } else {
                // FIXME: this is 'slow'
                for row in appids.iter() {
                    let (app_id, app_name) = row?;
                    let app_id = app_id
                        .as_ref()
                        .try_into()
                        .map_err(|err| anyhow!("Invalid database: {err}"))?;
                    let app_id = u32::from_le_bytes(app_id);
                    let app_name = std::str::from_utf8(app_name.as_ref())
                        .map_err(|err| anyhow!("Invalid database: {err}"))?;

                    if !regex.is_match(app_name) {
                        continue;
                    }

                    sorted.push((Reverse(app_name.to_string()), app_id));
                }
            }

            while let Some((Reverse(app_name), app_id)) = sorted.pop() {
                println!("{}: {}", app_name.bright_green(), app_id.yellow());
            }
        }
        CliArgs::CompatData(CliCompatData {
            case_sensitive,
            drive_c,
            regex,
        }) => {
            let appids = db::open_db().await?;

            let regex = RegexBuilder::new(regex.as_str())
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|err| anyhow!("App name regex must be valid: {err}"))?;

            let libs = library::load_steam_libraries()?;
            let mut games = libs
                .apps(&appids)
                .filter(|app| app.app_name().map_or(false, |name| regex.is_match(name)))
                .map(Reverse)
                .collect::<BinaryHeap<_>>();

            while let Some(Reverse(game)) = games.pop() {
                let mut path = game
                    .path
                    .join("steamapps")
                    .join("compatdata")
                    .join(format!("{}", game.app_id));

                if drive_c {
                    path = path.join("pfx").join("drive_c");
                }

                if simple {
                    print_path(&path);
                } else {
                    println!(
                        "{}: {}",
                        game.app_name().unwrap_or("").bright_green(),
                        path.display().yellow(),
                    );
                }
            }
        }
        CliArgs::Completions(CliCompletions { shell }) => generate(
            shell,
            &mut CliArgs::command(),
            env!("CARGO_BIN_NAME"),
            &mut io::stdout(),
        ),
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn print_path(path: &Path) {
    use os::unix::ffi::OsStrExt;
    use std::io::Write;
    _ = io::stdout().write_all(path.as_os_str().as_bytes());
    println!();
}

#[cfg(target_family = "windows")]
fn print_path(path: &Path) {
    // TODO:
    println!("{}", path.display());
}
