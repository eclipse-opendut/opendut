use anyhow::Context;
use serde::Deserialize;
use std::ops::Not;
use std::time::Duration;
use std::{fs, time};
use tracing::{debug, info};

pub const OS_LIST_URL: &str = "https://downloads.raspberrypi.com/os_list_imagingutility_v4.json";
pub const OS_LIST_ENTRY_OTHER: &str = "Raspberry Pi OS (other)";
pub const IMAGE_NAME: &str = "Raspberry Pi OS Lite (64-bit)";


pub fn determine_up_to_date_image() -> anyhow::Result<String> {
    debug!("Determining OS image to use.");

    let os_list_cache_file = super::temp_dir()?.join("os-index.json");

    let os_list_cache_text =
        if os_list_cache_file.exists().not()
        || time::SystemTime::now().duration_since(os_list_cache_file.metadata()?.modified()?)? > Duration::from_secs(60 * 60 * 24 * 7) { //older than 1 week
            debug!("Raspberry Pi OS list is missing or out-of-date. Downloading...");
            let os_list = reqwest::blocking::get(OS_LIST_URL)?
                .text()?;
            fs::write(os_list_cache_file, &os_list)?;
            os_list
        } else {
            fs::read_to_string(os_list_cache_file)?
        };

    let os_list: OsListJson = serde_json::from_str(&os_list_cache_text)?;

    let other = os_list.os_list.into_iter()
        .find(|entry| entry.name == OS_LIST_ENTRY_OTHER)
        .context(format!("List of operating system images does not contain entry with name '{OS_LIST_ENTRY_OTHER}'"))?;

    let image = other.subitems
        .context("Entry for other operating systems contains no subitems.")?
        .into_iter()
        .find(|entry| entry.name == IMAGE_NAME)
        .context(format!("List of operating system images does not contain entry with desired image name '{IMAGE_NAME}'"))?;

    let image = image.url
        .context("Entry for desired image does not contain a download URL.")?;

    info!("Selected OS image: {image}");

    Ok(image)
}

#[derive(Debug, Deserialize)]
struct OsListJson {
    os_list: Vec<OsListEntry>,
}

#[derive(Debug, Deserialize)]
struct OsListEntry {
    name: String,
    subitems: Option<Vec<OsListEntrySubitems>>,
}

#[derive(Debug, Deserialize)]
struct OsListEntrySubitems {
    name: String,
    url: Option<String>,
}
