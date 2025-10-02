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
certutil -A -n "localenv_ca" -t "CT,c" -i /provision/pki/opendut-ca.pem -d "$FIREFOX_PROFILE_PATH"

chown -R "$PUID":"$PGID" /config/.mozilla/firefox/


if [ ! -e "/config/.firstrun" ]; then
  echo "First run, opening a bunch of sites"
  touch /config/.firstrun
  # abc is the user firefox runs as
  su - abc -c 'DISPLAY=:1 firefox https://auth.opendut.local'
  su - abc -c 'DISPLAY=:1 firefox https://netbird.opendut.local'
  su - abc -c 'DISPLAY=:1 firefox https://carl.opendut.local'
  echo "Opened initial sites"
fi

# keep running to avoid constant re-run
sleep infinity
