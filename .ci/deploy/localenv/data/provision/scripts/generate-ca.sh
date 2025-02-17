#!/bin/sh

set -e

PROVISION_ROOT_DIR="/provision/"
PROVISION_PKI_DIR="$PROVISION_ROOT_DIR/pki/"
OPENDUT_PASSWORD_FILE="$PROVISION_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PROVISION_ROOT_DIR/.env"
CA_PATH="$PROVISION_PKI_DIR/opendut-ca"

mkdir -p "$PROVISION_PKI_DIR"

if [ ! -e "$OPENDUT_PASSWORD_FILE" ]; then
  echo "Password file $OPENDUT_PASSWORD_FILE missing. You may override the environment variable OPENDUT_PASSWORD_FILE."
  echo "Generate a password file with 'pwgen -s 50 1 > $OPENDUT_PASSWORD_FILE' or 'openssl rand -base64 50 > $OPENDUT_PASSWORD_FILE"
  exit 1
fi

if [ -e "$CA_PATH.key" ]; then
  echo "Refusing to overwrite existing ca."
  exit 1
fi

openssl genrsa -aes256 -out "$CA_PATH".key -passout file:"$OPENDUT_PASSWORD_FILE" 4096

# CA public key
openssl req -x509 -new -key "$CA_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -sha256 -days 9999 -outform PEM -out "$CA_PATH".pem -subj '/CN=Development CA/C=XX/ST=Some-State/O=ExampleOrg'
