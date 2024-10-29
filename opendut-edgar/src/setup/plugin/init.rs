use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};
use std::ffi::OsStr;
use std::ops::Not;
use crate::common::task::Task;
use crate::fs;
use crate::setup::plugin::plugin_runtime::PluginRuntime;

pub fn create_plugin_runtime(tasks: &mut Vec<Box<dyn Task>>) -> PluginRuntime {

    let plugin_runtime = PluginRuntime::new();
    let plugin_paths = discover_plugins().unwrap();

    let mut plugins: Vec<Box<dyn Task>> = plugin_paths.iter()
        .map(|path| Box::new(plugin_runtime.create_plugin_from_wasm(path)) as Box<dyn Task>)
        .collect();

    if plugins.is_empty().not() {
        tasks.append(&mut plugins);
    } else {
        debug!("No plugins loaded.");
    }

    plugin_runtime
}

fn discover_plugins() -> anyhow::Result<Vec<PathBuf>> {
	let path = super::constants::path_in_edgar_distribution()?;

    discover_plugins_in_path(&path)
}

fn discover_plugins_in_path(plugin_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !plugin_dir.exists() {
        warn!("File or folder '{}' does not exist.", plugin_dir.display());
        return anyhow::Ok(vec![]);
    }

    let plugin_order = read_plugin_order(plugin_dir)?;

    let mut plugin_paths: Vec<PathBuf> = vec![];

    for entry in plugin_order {
        if entry.is_dir() {
            plugin_paths.append(&mut discover_plugins_in_path(&entry)?);
        } else if entry.extension().and_then(OsStr::to_str) == Some("wasm") {
            debug!("Found plugin at: {}", entry.display());
            plugin_paths.push(entry);
        }
    }

    anyhow::Ok(plugin_paths)
}

fn read_plugin_order(plugin_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let config_path = plugin_dir.join("plugins.txt");

    if !config_path.exists() {
        warn!("No plugin configuration found at: {}", plugin_dir.display());
        return anyhow::Ok(vec![]);
    }

    let config = fs::read_to_string(&config_path)?;
    let potential_plugin_paths = config.lines()
        .map(|line| line.trim())
        .filter(|line| line.is_empty())
        .map(PathBuf::from);

    let mut paths: Vec<PathBuf> = vec![];

    for mut potential_path in potential_plugin_paths {
        if !potential_path.is_absolute() {
            potential_path = plugin_dir.join(&potential_path);
        }

        if potential_path == plugin_dir {
            warn!("The plugins.txt file at '{}' refers to itself. Ignoring this reference.", config_path.display());
            continue;
        }

        if potential_path.exists() {
            paths.push(potential_path);
        } else {
            error!("Plugin {} specified in {}/plugins.txt does not exist", potential_path.display(), plugin_dir.display());
        }
    };

    anyhow::Ok(paths)
}
