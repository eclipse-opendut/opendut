#!/bin/bash

source /netbird-api-functions.sh

netbird_auth
if [ -z "$TOKEN" ]; then
  echo "Failed to authenticate with Netbird"
  exit 1
fi

if [ -e "/management/api_key" ]; then
  API_KEY=$(cat /management/api_key)
  # check if api token works
  if netbird_api_token_test "$API_KEY"; then
    echo "API token works."
  else
    echo "API token does not work."
  fi
fi


curl --silent --fail -H "Authorization: Bearer $TOKEN" localhost/api/groups > /dev/null
exit $?
