//! Flash an SD card with a modified image for the Raspberry Pi to allow for easier bootstrapping.
//!
//! These modifications don't influence how openDuT is executed.

use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::ops::Not;
use std::path::Path;
use std::process::Command;
use indoc::{eprintdoc, formatdoc};
use crate::core::util::RunRequiringSuccess;

///Flash a device with an operating system image
#[derive(clap::Parser)]
pub struct FlashCli {
    /// The kind of device to flash
    #[command(subcommand)]
    device: DeviceKind,
}

#[derive(clap::Subcommand)]
enum DeviceKind {
    /// Flash an SD card for a Raspberry Pi
    #[command(alias="rpi", alias="raspi")]
    RaspberryPi {
        /// The storage device to flash, e.g. /dev/mmcblk0
        #[arg(long)]
        storage: String,

        /// The hostname to set on the Raspberry Pi
        #[arg(long)]
        hostname: String,

        /// The WiFi country code to use, e.g. US, DE, GB, FR.
        /// For a complete list, see: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
        #[arg(long)]
        wifi_country: String,
    },
}

const OS_LIST_URL: &str = "https://downloads.raspberrypi.com/os_list_imagingutility_v4.json";
const OS_LIST_ENTRY_OTHER: &str = "Raspberry Pi OS (other)";

const IMAGE_NAME: &str = "Raspberry Pi OS Lite (64-bit)";


// NetworkManager uses 10.42.x.0/24 for WiFi hotspots by default, where the 'x' is incremented for each wlan-interface.
// We assume that users don't have a WiFi dongle plugged in during setup, meaning there should only be wlan0, which will use x=0.
// And we can assume the last octet to be .1, because the Raspberry Pi serves as the gateway.
const SSH_ADDRESS: &str = "10.42.0.1";

const USERNAME: &str = "pi"; //encoded in firstrun.sh below
const PASSWORD: &str = "raspberry"; //encoded in firstrun.sh below; if you change this, run `mkpasswd $NEW_PASSWORD` to get the hash for the firstrun.sh
//TODO gitleaks

const SYSTEMD_SERVICE_NAME: &str = "opendut-setup-mode";
const SETUP_SCRIPT_PATH: &str = "/opt/opendut-setup-mode.sh";
const SETUP_COMPLETE_SCRIPT_PATH: &str = "/opt/opendut-setup-complete.sh";


impl FlashCli {
    #[tracing::instrument(name="flash", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.device {
            DeviceKind::RaspberryPi { storage: storage_to_flash, hostname, wifi_country } => {

                let image = Self::determine_image()?;

                let first_run_script_file = std::env::temp_dir().join("opendut-flash-raspberry-pi-firstrun.sh");
                let setup_params = SetupParams {
                    hostname: hostname.clone(),
                    wifi_country,
                };
                Self::template_first_run_script(&first_run_script_file, setup_params)?;

                Self::run_rpi_imager(&image, &storage_to_flash, &first_run_script_file)?;

                eprintdoc!(r#"

                    When you boot the Raspberry Pi, it spans a WiFi hotspot with the name "{hostname}".
                    You can configure the Raspberry Pi via SSH by connecting to this hotspot and then running:

                      ssh {USERNAME}@{SSH_ADDRESS}

                    The default password is "{PASSWORD}". Make sure to change it.

                    When you're done configuring, you want to run the {SETUP_COMPLETE_SCRIPT_PATH} script,
                    which will deactivate the hotspot and reboot the Raspberry Pi.
                "#);
            }
        }

        Ok(())
    }

    fn determine_image() -> anyhow::Result<String> {
        let os_list_cache_file = std::env::temp_dir().join("opendut-flash-raspberry-pi-os-index.json");

        let os_list_cache_text =
            if os_list_cache_file.exists().not() {
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

        Ok(image)
    }

    fn template_first_run_script(first_run_path: &Path, setup_params: SetupParams) -> anyhow::Result<()> {
        fs::write(first_run_path, setup_params.into_firstrun_script())?;
        Ok(())
    }

    fn run_rpi_imager(image: &str, storage_to_flash: &str, first_run_script_file: &Path) -> crate::Result {
        Command::new("rpi-imager")
            .arg("--cli")
            .arg("--first-run-script").arg(first_run_script_file)
            .arg(image)
            .arg(storage_to_flash)
            .run_requiring_success()?;

        Ok(())
    }
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

struct SetupParams {
    hostname: String,
    wifi_country: String,
}
impl SetupParams {
    fn into_firstrun_script(self) -> String {
        let setup_script = self.setup_mode_script();
        let setup_complete_script = self.setup_complete_script();

        let systemd_unit = Self::setup_mode_systemd_unit();

        formatdoc!(r#"
            #!/bin/bash
            set +e

            /usr/lib/userconf-pi/userconf '{USERNAME}' '$y$j9T$eXZlT3ZB7IG1JM4QKkRHx1$15c39m9beq/LRQuydVVDNG8b14MU.mt6RhHNydggzQ.'  # password "raspberry"

            echo '{setup_script}' > {SETUP_SCRIPT_PATH}
            chmod +x {SETUP_SCRIPT_PATH}

            echo '{setup_complete_script}' > {SETUP_COMPLETE_SCRIPT_PATH}
            chmod +x {SETUP_COMPLETE_SCRIPT_PATH}

            echo '{systemd_unit}' > /etc/systemd/system/{SYSTEMD_SERVICE_NAME}.service
            systemctl daemon-reload
            systemctl enable {SYSTEMD_SERVICE_NAME}
        "#)
    }

    fn setup_mode_script(&self) -> String {
        let SetupParams { hostname, wifi_country } = self;

        formatdoc!(r#"
            #!/bin/sh

            while true; do
                raspi-config nonint do_wifi_country {wifi_country}

                sleep 1  # wlan0 interface needs a moment after setting WiFi country

                nmcli device wifi hotspot \
                    ssid {hostname} \
                    password {PASSWORD}

                if [ $? -eq 0 ]; then
                    break
                else
                    echo "WiFi interface is still blocked after setting country. Retrying..."
                fi
            done

            systemctl enable ssh --now
        "#)
    }

    fn setup_mode_systemd_unit() -> String {
        formatdoc!(r#"
            [Unit]
            Wants=network-online.target

            [Service]
            Type=oneshot
            ExecStart={SETUP_SCRIPT_PATH}
            RemainAfterExit=true
            StandardOutput=journal

            [Install]
            WantedBy=multi-user.target
        "#)
    }

    fn setup_complete_script(&self) -> String {
        formatdoc!(r#"
            #!/bin/sh

            systemctl disable {SYSTEMD_SERVICE_NAME}
            reboot
        "#)
    }
}
