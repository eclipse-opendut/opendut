#!/bin/bash

source /netbird-api-functions.sh

netbird_auth
curl --silent --fail -H "Authorization: Bearer $TOKEN" "$NETBIRD_MANAGEMENT_URL"/api/groups > /dev/null
exit $?
