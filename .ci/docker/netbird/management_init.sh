#!/bin/bash

ls -la /usr/local/share/ca-certificates/
/usr/sbin/update-ca-certificates
source /netbird-api-functions.sh

wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
echo "Keycloak available"

wait_for_url "$KEYCLOAK_REALM_URL" 600 5 || exit 1
echo "Keycloak realm available: $KEYCLOAK_REALM_URL"

wait_for_keycloak_client_auth_successful 600 5 || exit 1
echo "Keycloak ready"

wait_for_keycloak_user__in_realm_netbird "netbird" 600 5 || exit 1
wait_for_keycloak_client__in_realm_netbird "netbird-backend" 600 5 || exit 1

exec /go/bin/netbird-mgmt management --port 80 --log-file console --disable-anonymous-metrics=true --single-account-mode-domain=netbird.opendut.local --dns-domain=netbird.opendut.local
