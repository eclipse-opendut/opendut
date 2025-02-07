use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};
use std::ffi::OsStr;
use std::ops::Not;
use crate::common::task::Task;
use crate::fs;
use crate::setup::plugin::plugin_runtime::PluginRuntime;

pub fn create_plugin_runtime(tasks: &mut Vec<Box<dyn Task>>) -> anyhow::Result<PluginRuntime> {

    let plugin_runtime = PluginRuntime::new();
    let plugins_dir = super::constants::path_in_edgar_distribution()?;
    let plugin_paths = discover_plugins_in_path(&plugins_dir)?;

    let mut plugins: Vec<Box<dyn Task>> = plugin_paths.iter()
        .map(|path| Box::new(plugin_runtime.create_plugin_from_wasm(path)) as Box<dyn Task>)
        .collect();

    if plugins.is_empty().not() {
        tasks.append(&mut plugins);
    } else {
        debug!("No plugins loaded.");
    }

    Ok(plugin_runtime)
}

fn discover_plugins_in_path(plugins_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if plugins_dir.exists().not() {
        warn!("No plugins folder found at: {}", plugins_dir.display());
        return anyhow::Ok(vec![]);
    }

    let plugin_order = read_plugin_order(plugins_dir)?;

    let mut plugin_paths: Vec<PathBuf> = vec![];

    for entry in plugin_order {
        if entry.is_dir() {
            plugin_paths.append(&mut discover_plugins_in_path(&entry)?);
        } else if entry.extension().and_then(OsStr::to_str) == Some("wasm") {
            debug!("Found plugin at: {}", entry.display());
            plugin_paths.push(entry);
        }
    }

    Ok(plugin_paths)
}

fn read_plugin_order(plugins_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let config_path = plugins_dir.join("plugins.txt");

    if config_path.exists().not() {
        warn!("No plugin configuration file found at: {}", config_path.display());
        return anyhow::Ok(vec![]);
    }

    let config = fs::read_to_string(&config_path)?;
    let potential_plugin_paths = config.lines()
        .map(|line| line.trim())
        .filter(|line| line.is_empty().not())
        .map(PathBuf::from);

    let mut paths: Vec<PathBuf> = vec![];

    for mut potential_path in potential_plugin_paths {

        if potential_path.is_absolute().not() {
            potential_path = plugins_dir.join(&potential_path);
        }

        if potential_path == plugins_dir {
            warn!("The plugins.txt file at '{}' refers to itself. Ignoring this reference.", config_path.display());
            continue;
        }

        if potential_path.exists() {
            paths.push(potential_path);
        } else {
            error!("Plugin {} specified in {} does not exist", potential_path.display(), config_path.display());
        }
    };

    Ok(paths)
}
