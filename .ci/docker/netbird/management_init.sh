#!/bin/bash

ls -la /usr/local/share/ca-certificates/
/usr/sbin/update-ca-certificates
source /netbird-api-functions.sh

wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
echo "Keycloak available"

wait_for_url "$KEYCLOAK_REALM_URL" 600 5 || exit 1
echo "Keycloak realm available"

wait_for_keycloak_client_auth_successful 600 5 || exit 1
echo "Keycloak ready"

wait_for_keycloak_client_in_realm_netbird netbird-backend

/go/bin/netbird-mgmt management --port 80 --log-file console --disable-anonymous-metrics=false --single-account-mode-domain=netbird.opendut.local --dns-domain=netbird.opendut.local
