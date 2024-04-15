#!/bin/sh

set -e

PKI_ROOT_DIR="$(dirname "$0")/store"
mkdir -p "$PKI_ROOT_DIR"
echo "PKI root directory: $PKI_ROOT_DIR"

CA_PATH="$PKI_ROOT_DIR/insecure-development-ca"
OPENDUT_PASSWORD_FILE="${OPENDUT_PASSWORD_FILE:-$HOME/.opendut-ca.pass}"
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
