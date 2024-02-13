#!/bin/bash

source /keycloak_functions.sh

main() {
  #set -ex

  echo "provisioning keycloak"
  wait_for_keycloak || { echo "Keycloak not ready, exiting"; exit 1;}
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

  # Create realm for opendut
  export REALM_OPENDUT="opendut"
  create_realm "$REALM_OPENDUT"

  # Create keycloak client for opendut-lea
  create_public_client "opendut-lea-client" '"https://carl/*"' "$REALM_OPENDUT"

  # Create keycloak client for opendut-carl
  create_secret_client "opendut-carl-client" "6754d533-9442-4ee6-952a-97e332eca38e" "$REALM_OPENDUT"

  # Create keycloak realm user groups
  create_realm_group testgroup "$REALM_OPENDUT"
  create_realm_group managergroup "$REALM_OPENDUT"

  # Create keycloak realm user roles
  create_realm_role testrole "$REALM_OPENDUT"
  create_realm_role managerrole "$REALM_OPENDUT"

  # Create keycloak realm test users: username, password, group, role
  create_user testuser testuser testgroup testrole "$REALM_OPENDUT"
  create_user opendut opendut testgroup testrole "$REALM_OPENDUT"
  create_user manager manager managergroup managerrole "$REALM_OPENDUT"
  # Create user nobody (no group, no role)
  create_user nobody nobody "" "" "$REALM_OPENDUT"

  # Provisioning done signal for opendut-theo
  echo "Keycloak provisioned"
  echo "0" > /opt/keycloak/data/provisioned
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
