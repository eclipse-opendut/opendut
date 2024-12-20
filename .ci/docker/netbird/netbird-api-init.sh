#!/bin/bash

source /netbird-api-functions.sh

# wait for keycloak
wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
# wait for realm to be provisioned
wait_for_url "$KEYCLOAK_URL/realms/netbird/.well-known/openid-configuration" 600 5 || exit 1
# wait for netbird to be ready
wait_for_url "$NETBIRD_MANAGEMENT_URL" 600 5 || exit 1
# wait for service response
wait_for_netbird_user_to_be_synced_from_keycloak "netbird" 600 5 || exit 1

netbird_auth

if [ ! -e "/management/api_key" ]; then
  API_KEY=$(get_netbird_api_token)
  if [ -n "$API_KEY" ]; then
    echo "$API_KEY" > /management/api_key
  else
    echo "Failed to retrieve API_KEY."
    #exit 1
  fi
else
  API_KEY=$(cat /management/api_key)
fi

echo API_KEY="$API_KEY"

# check if api token works
if netbird_api_token_test "$API_KEY"; then
  echo "API token works."
else
  echo "API token does not work."
fi

# Leave the container running
sleep infinity &
# Wait for any process to exit
wait -n
# Exit with status of process that exited first
exit $?
