#!/bin/bash

source /keycloak_functions.sh

main() {
  #set -ex

  echo "provisioning keycloak"
  wait_for_keycloak || { echo "Keycloak not ready, exiting"; exit 1;}
  kcauth

  # https://docs.netbird.io/selfhosted/identity-providers#keycloak
  create_realm netbird
  create_user netbird "$NETBIRD_PASSWORD" "" "" "netbird"
  # frontend client NETBIRD_AUTH_CLIENT_ID
  create_public_client "netbird-client" '"https://netbird.opendut.local/*"' "netbird"
  create_client_scope "api" "default" "netbird"
  client_scope_add_audience "api" "Audience for NetBird Management API" "netbird-client" "netbird"
  add_client_scope_to_client "netbird-client" "api" "netbird"

  # CLI client for provisioning netbird API
  create_public_client_with_direct_access "netbird-mgmt-cli" '"https://netbird.opendut.local/*"' "netbird" "https://netbird.opendut.local"
  add_client_scope_to_client "netbird-mgmt-cli" "api" "netbird"

  # backend client NETBIRD_IDP_MGMT_CLIENT_ID, NETBIRD_IDP_MGMT_CLIENT_SECRET
  create_secret_client "netbird-backend" "$NETBIRD_MANAGEMENT_CLIENT_SECRET" "netbird"
  client__assign_service_account_role "netbird-backend" "realm-management" "view-users" "netbird"

  # Create realm for opendut
  export REALM_OPENDUT="opendut"
  create_realm "$REALM_OPENDUT"

  # Create keycloak client for opendut-lea
  create_public_client "opendut-lea-client" '"https://carl.opendut.local/*"' "$REALM_OPENDUT"

  # groups scope
  GROUP_SCOPE_NAME="groups"
  create_client_scope "$GROUP_SCOPE_NAME" "default" "$REALM_OPENDUT"
  create_client_scope_groups "$GROUP_SCOPE_NAME" "$REALM_OPENDUT"
  add_client_scope_to_client "opendut-lea-client" "$GROUP_SCOPE_NAME" "$REALM_OPENDUT"
  update_existing_client_scope_realm_roles "roles" "$REALM_OPENDUT"

  # Create keycloak client for opendut-carl
  create_secret_client "opendut-carl-client" "$OPENDUT_CARL_NETWORK_OIDC_CLIENT_SECRET" "$REALM_OPENDUT"

  # Create keycloak realm user groups
  create_realm_group testgroup "$REALM_OPENDUT"
  create_realm_group managergroup "$REALM_OPENDUT"

  # Create keycloak realm user roles
  create_realm_role testrole "$REALM_OPENDUT"
  create_realm_role managerrole "$REALM_OPENDUT"

  # Create keycloak realm test users: username, password, group, role
  create_user opendut "$OPENDUT_USER_OPENDUT_PASSWORD" testgroup testrole "$REALM_OPENDUT"

  # Create keycloak client for opendut CLIs CLEO/EDGAR
  create_secret_client "opendut-cleo-client" "$OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET" "$REALM_OPENDUT"
  create_secret_client "opendut-edgar-client" "$OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET" "$REALM_OPENDUT"

  # Create keycloak client privileges for openDuT-CARL
  create_realm_role carl-admin "$REALM_OPENDUT"
  # Add role carl-admin to client opendut-carl-client
  kcadm add-roles -r "$REALM_OPENDUT" --uusername service-account-opendut-carl-client --rolename carl-admin
  # Add keycloak-internal-role realm-admin to realm-role carl-admin. This is an all-powerful role, use with caution.
  make_realm_role_admin "carl-admin" "$REALM_OPENDUT"

  # Provisioning done signal for opendut-theo
  echo "Keycloak provisioned"
  create_realm "provisioning-done"
  echo "0" > /opt/keycloak/data/provisioned
}


# update certificates
# TODO: move update custom certificates to entrypoint

main

if [ -n "$1" ]; then
  # Leave the container running if an argument is provided
  exec sleep infinity &

  # Wait for any process to exit
  wait -n

  # Exit with status of process that exited first
  exit $?
fi
