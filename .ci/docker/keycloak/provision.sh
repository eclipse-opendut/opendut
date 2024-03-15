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
  create_public_client "netbird-client" '"https://netbird-ui/*", "https://netbird-dashboard/*"' "netbird"
  create_client_scope "api" "default" "netbird"
  client_scope_add_audience "api" "Audience for NetBird Management API" "netbird-client"
  add_client_scope_to_client "netbird-client" "api" "netbird"

  # CLI client for provisioning netbird API
  create_public_client_with_direct_access "netbird-mgmt-cli" '"https://netbird-ui/*", "https://netbird-dashboard/*"' "netbird"
  add_client_scope_to_client "netbird-mgmt-cli" "api" "netbird"

  # backend client NETBIRD_IDP_MGMT_CLIENT_ID, NETBIRD_IDP_MGMT_CLIENT_SECRET
  create_secret_client "netbird-backend" "871adc12-8a09-11ee-ab98-fb08e2456f00" "netbird"
  client__assign_service_account_role "netbird-backend" "realm-management" "view-users"

  # Create realm for opendut
  export REALM_OPENDUT="opendut"
  create_realm "$REALM_OPENDUT"

  # Create keycloak client for opendut-lea
  create_public_client "opendut-lea-client" '"https://carl/*"' "$REALM_OPENDUT"

  # groups scope
  GROUP_SCOPE_NAME="groups"
  create_client_scope "$GROUP_SCOPE_NAME" "default" "$REALM_OPENDUT"
  create_client_scope_groups "$GROUP_SCOPE_NAME" "$REALM_OPENDUT"
  add_client_scope_to_client "opendut-lea-client" "$GROUP_SCOPE_NAME" "$REALM_OPENDUT"
  update_existing_client_scope_realm_roles "roles" "$REALM_OPENDUT"

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

  # Create keycloak client for opendut CLIs CLEO/EDGAR
  create_secret_client "opendut-cleo-client" "918642e0-4ec4-4ef5-8ae0-ba92de7da3f9" "$REALM_OPENDUT"
  create_secret_client "opendut-edgar-client" "c7d6ace0-b90f-471a-bb62-a4ecac4150f8" "$REALM_OPENDUT"

  # Create keycloak client privileges for openDuT-CARL
  create_realm_role carl-admin "$REALM_OPENDUT"
  # Add role carl-admin to client opendut-carl-client
  kcadm add-roles -r "$REALM_OPENDUT" --uusername service-account-opendut-carl-client --rolename carl-admin
  # Add keycloak-internal-role realm-admin to realm-role carl-admin. This is an all-powerful role, use with caution.
  make_realm_role_admin "carl-admin" "$REALM_OPENDUT"


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
