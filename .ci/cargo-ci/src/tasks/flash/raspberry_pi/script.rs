use indoc::formatdoc;

use super::{PASSWORD, USERNAME};

const SYSTEMD_SERVICE_NAME: &str = "opendut-setup-mode";
const SETUP_SCRIPT_PATH: &str = "/opt/opendut-setup-mode.sh";
pub(super) const SETUP_COMPLETE_SCRIPT_PATH: &str = "/opt/opendut-setup-complete.sh";


pub struct Params {
    pub hostname: String,
    pub wifi_country: String,
}

pub fn format_first_run_script(params: Params) -> String {
    let setup_script = setup_mode_script(params);
    let setup_complete_script = setup_complete_script();

    let systemd_unit = setup_mode_systemd_unit();

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

fn setup_mode_script(params: Params) -> String {
    let Params { hostname, wifi_country } = params;

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

fn setup_complete_script() -> String {
    formatdoc!(r#"
        #!/bin/sh

        systemctl disable {SYSTEMD_SERVICE_NAME}
        reboot
    "#)
}
