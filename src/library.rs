use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Result};
use directories::UserDirs;
use sled::{IVec, Tree};

use crate::vdf;

//

#[derive(Debug)]
pub struct SteamLibraries {
    // one steam library location, like `~/games/`, anything that has `libraryfolder.vdf` in it
    libs: Vec<SteamLibrary>,
}

impl SteamLibraries {
    // pub fn app_ids(&self) -> impl Iterator<Item = u32> + '_ {
    //     self.libs.iter().flat_map(|lib| lib.games.iter()).copied()
    // }

    pub fn apps<'a>(&'a self, appids: &'a Tree) -> impl Iterator<Item = SteamApp<'_>> + 'a {
        self.libs.iter().flat_map(|lib| {
            lib.games.iter().filter_map(|id| {
                Some(SteamApp {
                    app_id: *id,
                    app_name: appids.get(id.to_le_bytes()).ok()??,
                    path: &lib.path,
                })
            })
        })
    }
}

//

#[derive(Debug)]
pub struct SteamApp<'a> {
    pub app_id: u32,
    app_name: IVec,
    pub path: &'a Path,
}

impl SteamApp<'_> {
    pub fn app_name(&self) -> Option<&str> {
        std::str::from_utf8(self.app_name.as_ref()).ok()
    }
}

impl PartialEq for SteamApp<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.app_name().eq(&other.app_name())
    }
}

impl Eq for SteamApp<'_> {}

impl PartialOrd for SteamApp<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SteamApp<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.app_name().cmp(&other.app_name())
    }
}

//

pub fn load_steam_libraries() -> Result<SteamLibraries> {
    let spinner =
        indicatif::ProgressBar::new_spinner().with_message("Loading Steam libraryfolders.vdf");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let base_steam = base_steam()?;
    let library_folders = base_steam.join("config").join("libraryfolders.vdf");

    let library_folders = fs::read_to_string(library_folders)
        .map_err(|err| anyhow!("Could not read Steam libraryfolders.vdf: {err}"))?;

    let library_folders = vdf::VdfParser::from_str(&library_folders)
        .parse_entries()
        .ok_or_else(|| anyhow!("Steam libraryfolders.vdf is invalid"))?;

    let libs = library_folders
        .get("libraryfolders")
        .ok_or_else(|| anyhow!("Steam libraryfolders.vdf: missing libraryfolders"))?
        .as_map()
        .ok_or_else(|| anyhow!("Steam libraryfolders.vdf: libraryfolders should be a map"))?
        .values()
        .filter_map(|library| {
            let library = library.as_map()?;
            let path = library.get("path")?.as_value()?.parse().ok()?;
            let games = library
                .get("apps")?
                .as_map()?
                .keys()
                .filter_map(|key| key.parse::<u32>().ok())
                .collect();

            Some(SteamLibrary { path, games })
        })
        .collect::<Vec<_>>();

    spinner.finish_with_message("Finished loading Steam libraryfolders.vdf");

    Ok(SteamLibraries { libs })
}

//

#[derive(Debug)]
struct SteamLibrary {
    path: PathBuf,
    games: HashSet<u32>,
}

fn base_steam() -> Result<PathBuf> {
    let dirs = UserDirs::new().ok_or_else(|| anyhow!("No valid home directory found"))?;
    // TODO: fix this hardcoded shit
    let base_steam_0 = dirs.home_dir().join(".steam").join("steam");
    let base_steam_1 = dirs.home_dir().join(".local").join("share").join("Steam");

    if base_steam_0.is_symlink() || base_steam_0.is_dir() {
        Ok(base_steam_0)
    } else if base_steam_1.is_symlink() || base_steam_1.is_dir() {
        Ok(base_steam_1)
    } else {
        Err(anyhow!("Base Steam dir not found"))
    }
}
