//! Flash an SD card with a modified image for the Raspberry Pi to allow for easier bootstrapping.
//!
//! These modifications don't influence how openDuT is executed.

mod image;
mod script;

use indoc::eprintdoc;
use script::SETUP_COMPLETE_SCRIPT_PATH;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::anyhow;
use tracing::{debug, info};

// NetworkManager uses 10.42.x.0/24 for WiFi hotspots by default, where the 'x' is incremented for each wlan-interface.
// We assume that users don't have a WiFi dongle plugged in during setup, meaning there should only be wlan0, which will use x=0.
// And we can assume the last octet to be .1, because the Raspberry Pi serves as the gateway.
const SSH_ADDRESS: &str = "10.42.0.1";

const USERNAME: &str = "pi";
const PASSWORD: &str = "raspberry"; //temporary password for initial setup; if you change this, run `mkpasswd $NEW_PASSWORD` and put the hash into script::format_first_run_script


/// Flash an SD card for a Raspberry Pi, so that it spans a WiFi hotspot for setting it up further.
///
/// This requires Raspberry Pi Imager to be installed (`rpi-imager` should be available on the operating system PATH).
#[derive(clap::Parser)]
#[command(alias="rpi", alias="raspi")]
pub struct RaspberryPiCli {
    /// The storage device to flash, e.g. /dev/mmcblk0
    #[arg(long)]
    storage: String,

    /// The name of the WiFi (SSID) to use for the hotspot on the Raspberry Pi
    #[arg(long)]
    wifi_name: String,

    /// The WiFi country code to use for the hotspot, e.g. US, DE, GB, FR.
    /// For a complete list, see: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
    #[arg(long)]
    wifi_country: String,
}
impl RaspberryPiCli {
    pub fn run(self) -> anyhow::Result<()> {
        let RaspberryPiCli { storage: storage_to_flash, wifi_name, wifi_country } = self;

        let image = image::determine_up_to_date_image()?;

        let first_run_script_path = temp_dir()?.join("firstrun.sh");

        Self::template_first_run_script(
            &first_run_script_path,
            script::Params {
                wifi_name: wifi_name.clone(),
                wifi_country,
            }
        )?;

        Self::run_rpi_imager(&image, &storage_to_flash, &first_run_script_path)?;

        eprintdoc!(r#"

            When you boot the Raspberry Pi, it spans a WiFi hotspot with the name "{wifi_name}".
            You can configure the Raspberry Pi via SSH by connecting to this hotspot and then running:

              ssh {USERNAME}@{SSH_ADDRESS}

            The default password is "{PASSWORD}". Make sure to change it.

            When you're done configuring, you want to run the {SETUP_COMPLETE_SCRIPT_PATH} script,
            which will deactivate the hotspot and reboot the Raspberry Pi.
        "#);

        Ok(())
    }

    fn template_first_run_script(first_run_path: &Path, params: script::Params) -> anyhow::Result<()> {
        debug!("Templating {first_run_path:?} for passing into rpi-imager.");
        fs::write(
            first_run_path,
            script::format_first_run_script(params)
        )?;
        Ok(())
    }

    fn run_rpi_imager(image: &str, storage_to_flash: &str, first_run_script_file: &Path) -> anyhow::Result<()> {
        info!("Running rpi-imager...");

        let mut command = Command::new("rpi-imager");

        command
            .arg("--cli")
            .arg("--first-run-script").arg(first_run_script_file)
            .arg(image)
            .arg(storage_to_flash);

        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Error while running {command:?}."))
        }
    }
}

fn temp_dir() -> anyhow::Result<PathBuf> {
    let path = std::env::temp_dir()
        .join("opendut-flash")
        .join("raspberry-pi");

    fs::create_dir_all(&path)?;

    Ok(path)
}
