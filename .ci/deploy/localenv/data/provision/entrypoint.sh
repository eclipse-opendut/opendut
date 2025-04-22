#!/bin/bash

create_password() {
  length="${1:-32}"
  # password="$(openssl rand -base64 "$length")"
  # pwgen produces more readable passwords
  password="$(pwgen "$length")"
  echo "$password"
}

PROVISION_ROOT_DIR="/provision/"
PROVISION_PKI_DIR="$PROVISION_ROOT_DIR/pki/"
OPENDUT_PASSWORD_FILE="$PROVISION_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PROVISION_ROOT_DIR/.env"
CA_PATH="$PROVISION_PKI_DIR/opendut-ca"

if [ ! -e "$OPENDUT_PASSWORD_FILE" ]; then
  mkdir "$PROVISION_PKI_DIR"
  # PKI
  pwgen --secure 32 1 > "$OPENDUT_PASSWORD_FILE"
  /scripts/recreate.sh noprompt

  # Keycloak
  echo KEYCLOAK_POSTGRES_PASSWORD="$(create_password)" >> $OPENDUT_ENV_FILE
  echo KEYCLOAK_ADMIN_PASSWORD="$(create_password)" >> $OPENDUT_ENV_FILE

  # Netbird
  echo NETBIRD_DATASTORE_ENC_KEY="$(create_password)" >> $OPENDUT_ENV_FILE
  echo TURN_PASSWORD="$(create_password)" >> $OPENDUT_ENV_FILE
  echo NETBIRD_MANAGEMENT_CLIENT_SECRET="$(create_password)" >> $OPENDUT_ENV_FILE
  echo NETBIRD_PASSWORD="$(create_password 16)" >> $OPENDUT_ENV_FILE

  # CARL / Keycloak
  echo OPENDUT_CARL_NETWORK_OIDC_CLIENT_SECRET="$(create_password)" >> $OPENDUT_ENV_FILE
  EDGAR_SECRET="$(create_password)"
  echo OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET="$EDGAR_SECRET" >> $OPENDUT_ENV_FILE
  echo OPENDUT_CARL_NETWORK_OIDC_CLIENT_PEER_SECRET="$EDGAR_SECRET" >> $OPENDUT_ENV_FILE
  echo OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET="$(create_password)" >> $OPENDUT_ENV_FILE
  echo OPENDUT_USER_OPENDUT="$(create_password 16)" >> $OPENDUT_ENV_FILE

  # Telemetry / Grafana
  echo GF_SECURITY_ADMIN_PASSWORD="$(create_password)" >> $OPENDUT_ENV_FILE
  echo GF_SECURITY_SECRET_KEY="$(create_password)" >> $OPENDUT_ENV_FILE

fi

chown -R carl:carl $PROVISION_ROOT_DIR
chmod a=,u=rw $OPENDUT_PASSWORD_FILE
chmod a=,u=rw $OPENDUT_ENV_FILE

# '/provision' - contains the created secrets in the docker volume
# '/secrets'   - contains a synchronized copy of the created secrets available on the host system
rsync --archive --delete $PROVISION_ROOT_DIR/ /secrets/
