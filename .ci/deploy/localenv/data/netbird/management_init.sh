#!/bin/bash

set -e # Exit on error

echo "Management init script for local test environment"

source /netbird-api-functions.sh

echo "Creating /etc/netbird/management.json from template"
envsubst </config/management.json.tmpl | jq . >/etc/netbird/management.json

wait_for_keycloak_client_auth_successful 600 5 || exit 1
echo "Keycloak ready"

wait_for_keycloak_user__in_realm_netbird "netbird" 600 5 || exit 1
wait_for_keycloak_client__in_realm_netbird "netbird-backend" 600 5 || exit 1


exec /go/bin/netbird-mgmt management --port 80 --log-file console --disable-anonymous-metrics=false --single-account-mode-domain="$NETBIRD_MANAGEMENT_DOMAIN" --dns-domain="$NETBIRD_MANAGEMENT_DOMAIN"
