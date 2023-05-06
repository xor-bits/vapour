use clap::{Args, Parser};
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use regex::RegexBuilder;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

//

mod vdf;

//

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
enum CliArgs {
    AppId(CliAppId),
    CompatData(CliCompatData),
}

#[derive(Debug, Clone, Copy, Args)]
struct CliAppId {
    app_id: u32,
}

#[derive(Debug, Clone, Args)]
struct CliCompatData {
    /// Make the regex case sensitive, by default it isn't
    #[clap(short, long)]
    case_sensitive: bool,

    /// Sort the output by game name
    #[clap(short, long)]
    sort: bool,

    /// basically appends "pfx/drive_c" to results
    #[clap(short, long)]
    drive_c: bool,

    /// Regex pattern for game names
    regex: Option<String>,
}

#[derive(Debug)]
struct SteamLibrary {
    path: PathBuf,
    games: HashSet<u32>,
}

#[derive(Debug)]
struct SteamApp<'a> {
    appid: u32,
    name: &'a str,
    path: &'a Path,
}

//

static DIRS: Lazy<ProjectDirs> = Lazy::new(|| {
    if let Some(dirs) = ProjectDirs::from("app", "xor-bits", "vapour") {
        return dirs;
    }

    panic!()
});

static ARGS: Lazy<CliArgs> = Lazy::new(CliArgs::parse);

static APP_ID_DB: Lazy<HashMap<u32, String>> = Lazy::new(|| {
    fs::create_dir_all(DIRS.cache_dir()).unwrap();
    let db_cache_file = DIRS.cache_dir().join("steam_appids");

    tracing::debug!("db cache file: {}", db_cache_file.display());

    let mut db_cache_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(db_cache_file)
        .unwrap();

    // TODO: file locking
    if db_cache_file.metadata().unwrap().len() == 0 {
        #[derive(Deserialize)]
        struct RawData {
            applist: RawApplist,
        }

        #[derive(Deserialize)]
        struct RawApplist {
            apps: Vec<RawApp>,
        }

        #[derive(Deserialize)]
        struct RawApp {
            appid: u32,
            name: String,
        }

        const URL: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

        tracing::debug!("Downloading Steam appid list from `{URL}`");

        let result: RawData = reqwest::blocking::get(URL).unwrap().json().unwrap();

        tracing::debug!("Download complete");

        let db: HashMap<u32, String> = result
            .applist
            .apps
            .into_iter()
            .map(|app| (app.appid, app.name))
            .collect();

        tracing::debug!("Writing DB file");

        bincode::serialize_into(&mut db_cache_file, &db).unwrap();
        // serde_json::ser::to_writer(&mut db_cache_file, &db).unwrap();

        tracing::debug!("DB loaded");

        db
    } else {
        tracing::debug!("Reading + deserializing DB file");

        let db = bincode::deserialize_from(&mut db_cache_file).unwrap();
        // let db = serde_json::de::from_reader(&mut db_cache_file).unwrap();

        tracing::debug!("DB loaded");

        db
    }
});

//

fn main() {
    tracing_subscriber::fmt::init();

    match &*ARGS {
        CliArgs::AppId(CliAppId { app_id }) => {
            let app = APP_ID_DB.get(app_id).unwrap();

            println!("{}:\n - {}", app_id.bright_green(), app.yellow());
        }
        CliArgs::CompatData(CliCompatData {
            case_sensitive,
            sort,
            drive_c,
            regex,
        }) => {
            let regex = regex.as_ref().map(|regex| {
                RegexBuilder::new(regex)
                    .case_insensitive(!*case_sensitive)
                    .build()
                    .unwrap()
            });

            let base_steam = base_steam();
            let library_folders = base_steam.join("config").join("libraryfolders.vdf");

            let library_folders = fs::read_to_string(library_folders).unwrap();

            let library_folders = vdf::VdfParser::from_str(&library_folders)
                .parse_entries()
                .unwrap();

            // TODO: fix this holy fucking unwrap hell
            let steam_libraries = library_folders
                .get("libraryfolders")
                .unwrap()
                .as_map()
                .unwrap()
                .values()
                .map(|library| {
                    let library = library.as_map().unwrap();
                    let path = library
                        .get("path")
                        .unwrap()
                        .as_value()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let games = library
                        .get("apps")
                        .unwrap()
                        .as_map()
                        .unwrap()
                        .keys()
                        .map(|key| key.parse::<u32>().unwrap())
                        .collect();

                    SteamLibrary { path, games }
                })
                .collect::<Vec<_>>();

            let mut games = steam_libraries
                .iter()
                .flat_map(|library| {
                    library
                        .games
                        .iter()
                        .copied()
                        .filter_map(|appid| {
                            Some(SteamApp {
                                appid,
                                name: APP_ID_DB.get(&appid)?,
                                path: &library.path,
                            })
                        })
                        .filter(|game| {
                            if let Some(regex) = regex.as_ref() {
                                regex.is_match(game.name)
                            } else {
                                true
                            }
                        })
                })
                .collect::<Vec<_>>();

            if *sort {
                games.sort_by_key(|game| game.name);
            }

            for game in games {
                let mut path = game
                    .path
                    .join("steamapps")
                    .join("compatdata")
                    .join(format!("{}", game.appid));

                if *drive_c {
                    path = path.join("pfx").join("drive_c");
                }

                println!(
                    "{}:\n - {}",
                    game.name.bright_green(),
                    path.display().yellow(),
                );
            }
        }
    }
}

fn base_steam() -> PathBuf {
    let dirs = UserDirs::new().unwrap();
    // TODO: fix this hardcoded shit
    let base_steam_0 = dirs.home_dir().join(".steam").join("steam");
    let base_steam_1 = dirs.home_dir().join(".local").join("share").join("Steam");

    if base_steam_0.is_symlink() || base_steam_0.is_dir() {
        base_steam_0
    } else if base_steam_1.is_symlink() || base_steam_1.is_dir() {
        base_steam_1
    } else {
        panic!("Base Steam dir not found")
    }
}
