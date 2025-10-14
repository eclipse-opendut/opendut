#!/bin/bash

source /opt/carl_functions.sh

echo "OpenDuT-CARL init script for local test environment"

# wait for keycloak
wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
# wait for realm to be provisioned
wait_for_url "$KEYCLOAK_URL/realms/netbird/.well-known/openid-configuration" 600 5 || exit 1
# wait for netbird to be ready
wait_for_url "$NETBIRD_MANAGEMENT_URL" 600 5 || exit 1
# wait for service response
wait_for_netbird_user_to_be_synced_from_keycloak "netbird" 600 5 || exit 1

exec "$@"
