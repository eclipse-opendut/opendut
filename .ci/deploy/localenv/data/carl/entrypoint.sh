#!/bin/bash

source /opt/carl_functions.sh

echo "OpenDuT-CARL init script for local test environment"

API_KEY_FILE="/opt/opendut-carl/config/api_key"

# wait for keycloak
wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
# wait for realm to be provisioned
wait_for_url "$KEYCLOAK_URL/realms/netbird/.well-known/openid-configuration" 600 5 || exit 1
# wait for netbird to be ready
wait_for_url "$NETBIRD_MANAGEMENT_URL" 600 5 || exit 1
# wait for service response
wait_for_netbird_user_to_be_synced_from_keycloak "netbird" 600 5 || exit 1

netbird_auth

if [ ! -e "$API_KEY_FILE" ]; then
  API_KEY=$(get_netbird_api_token)
  if [ -n "$API_KEY" ]; then
    echo "$API_KEY" > $API_KEY_FILE
  else
    echo "Failed to retrieve API_KEY."
  fi
else
  API_KEY=$(cat $API_KEY_FILE)
fi
export OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET=$API_KEY

exec "$@"
