#!/bin/bash

source /netbird-api-functions.sh

netbird_auth
if [ -z "$TOKEN" ]; then
  echo "Failed to authenticate with NetBird"
  exit 1
fi

curl --silent --fail -H "Authorization: Bearer $TOKEN" localhost/api/groups > /dev/null
exit $?
