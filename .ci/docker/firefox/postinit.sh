#!/bin/bash

# This is a post init script that runs after the entrypoint.sh script.in background
# It adds custom certificates to the firefox profile once firefox is available

# s6-overlay hints:
# How to run any post-service scripts? #185
# https://github.com/just-containers/s6-overlay/issues/185

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
