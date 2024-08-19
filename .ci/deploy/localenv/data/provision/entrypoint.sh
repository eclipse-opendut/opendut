#!/bin/bash

create_password() {
  length="${1:-32}"
  # password="$(openssl rand -base64 "$length")"
  # pwgen produces more readable passwords
  password="$(pwgen "$length")"
  echo "$password"
}

PKI_ROOT_DIR="/provision/"
OPENDUT_PASSWORD_FILE="$PKI_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PKI_ROOT_DIR/.env"
if [ ! -e "$OPENDUT_PASSWORD_FILE" ]; then
  mkdir "/provision/store"
  # PKI
  pwgen --secure 32 1 > "$OPENDUT_PASSWORD_FILE"
  "$PKI_ROOT_DIR"/recreate.sh noprompt

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

fi

chown -R carl:carl /provision
cp $OPENDUT_ENV_FILE /secrets/.env
rsync --archive --delete /provision/store/ /secrets/pki/
