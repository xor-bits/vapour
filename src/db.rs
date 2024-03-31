use std::{fs, time::Duration};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use sled::Tree;

//

pub async fn update_appids(appids: &Tree) -> Result<()> {
    let raw = download_database()
        .await
        .map_err(|err| anyhow!("Failed to download the appid database: {err}"))?;

    let spinner = indicatif::ProgressBar::new_spinner().with_message("Updating database");
    spinner.enable_steady_tick(Duration::from_millis(100));

    appids.clear()?;
    for app in raw.applist.apps {
        appids.insert(app.appid.to_le_bytes(), app.name.as_bytes())?;
    }

    spinner.finish_with_message("Finished updating database");

    Ok(())
}

pub async fn open_db() -> Result<Tree> {
    let spinner = indicatif::ProgressBar::new_spinner().with_message("Opening database");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let dirs = ProjectDirs::from("app", "xor-bits", "vapour")
        .ok_or_else(|| anyhow!("No valid home directory found"))?;

    fs::create_dir_all(dirs.cache_dir())
        .map_err(|err| anyhow!("Invalid cache directory: {err}"))?;

    let db = sled::open(dirs.cache_dir().join("vapour.db"))
        .map_err(|err| anyhow!("Invalid database: {err}"))?;

    let appids = db.open_tree("appids")?;

    spinner.finish_with_message("Finished opening database");

    // initial download of the appid database
    if db.insert("updated", "")?.is_none() {
        update_appids(&appids).await?;
    }

    Ok(appids)
}

//

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

//

async fn download_database() -> Result<RawData> {
    const URL: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

    let resp = reqwest::get(URL).await?;

    let spinner = indicatif::ProgressBar::new_spinner()
        .with_message(format!("Downloading appid list from {URL}"));
    spinner.enable_steady_tick(Duration::from_millis(100));

    let json = resp.json().await;

    spinner.finish_with_message(format!("Finished downloading appid list from {URL}"));
    Ok(json?)

    // TODO: progress bar
    // let Some(total) = resp.content_length() else {
    //     // no size -> no loading bar

    // };

    // let bar = indicatif::ProgressBar::new(total);
    // bar.set_message(format!("Downloading appid list from {URL}"));

    // let mut stream = resp.bytes_stream();
    // let mut result = vec![];
    // let mut downloaded = 0;

    // while let Some(chunk) = stream.next().await {
    //     let chunk = chunk?;
    //     result.extend_from_slice(chunk.as_ref());
    //     downloaded = total.min(downloaded + chunk.len() as u64);
    //     bar.set_position(downloaded);

    //     tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    // }

    // bar.finish_with_message("Downloaded appid list");

    // let result: RawData = serde_json::de::from_slice(&result)?;

    // Ok(result)
}
