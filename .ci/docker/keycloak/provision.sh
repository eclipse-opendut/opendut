#!/bin/bash

source /keycloak_functions.sh

main() {
  #set -ex

  echo "provisioning keycloak"
  wait_for_keycloak
  kcauth

  # https://docs.netbird.io/selfhosted/identity-providers#keycloak
  create_realm netbird
  create_user netbird netbird "" ""
  # frontend client NETBIRD_AUTH_CLIENT_ID
  create_public_client "netbird-client" '"http://netbird-ui/*", "http://netbird-dashboard/*"' "netbird"
  create_client_scope "api" "default" "netbird"
  client_scope_add_audience "api" "Audience for NetBird Management API" "netbird-client"
  add_client_scope_to_client "netbird-client" "api" "netbird"

  # CLI client for provisioning netbird API
  create_public_client_with_direct_access "netbird-mgmt-cli" '"http://netbird-ui/*", "http://netbird-dashboard/*"' "netbird"
  add_client_scope_to_client "netbird-mgmt-cli" "api" "netbird"

  # backend client NETBIRD_IDP_MGMT_CLIENT_ID, NETBIRD_IDP_MGMT_CLIENT_SECRET
  create_secret_client "netbird-backend" "871adc12-8a09-11ee-ab98-fb08e2456f00" "netbird"
  client__assign_service_account_role "netbird-backend" "realm-management" "view-users"
  echo "Keycloak provisioned"

}

main

if [ -n "$1" ]; then
  # Leave the container running if an argument is provided
  sleep infinity &

  # Wait for any process to exit
  wait -n

  # Exit with status of process that exited first
  exit $?
fi
