#!/usr/bin/with-contenv bash

# https://docs.linuxserver.io/general/container-customization/#custom-services

# This script configures the firefox profile
# - It runs as custom service at the same time as firefox
# - Adds custom certificates to firefox profile
# - On first startup opens known pages in opendut
# - Keeps running to avoid re-run

echo "Waiting for firefox profile to exist"
sleep 5
echo "Adding certificates to firefox profile"

FIREFOX_PROFILE_PATH=$(find /config/.mozilla/firefox/ -iname '*.default-release')
# creates empty cert.db
# certutil -N --empty-password -d sql:$FIREFOX_PROFILE_PATH
certutil -L -d "$FIREFOX_PROFILE_PATH"
certutil -A -n "custom_ca1" -t "CT,c" -i /usr/local/share/ca-certificates/opendut_custom_ca_OPENDUT_CUSTOM_CA1.crt -d "$FIREFOX_PROFILE_PATH"
certutil -A -n "custom_ca2" -t "CT,c" -i /usr/local/share/ca-certificates/opendut_custom_ca_OPENDUT_CUSTOM_CA2.crt -d "$FIREFOX_PROFILE_PATH"

chown -R "$PUID":"$PGID" /config/.mozilla/firefox/


if [ ! -e "/config/.firstrun" ]; then
  echo "First run, opening a bunch of sites"
  touch /config/.firstrun
  # abc is the user firefox runs as
  su - abc -c 'DISPLAY=:1 firefox https://keycloak'
  su - abc -c 'DISPLAY=:1 firefox https://netbird-dashboard'
  su - abc -c 'DISPLAY=:1 firefox https://carl'
fi

# keep running to avoid constant re-run
sleep infinity
