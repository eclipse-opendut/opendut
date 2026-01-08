#!/bin/bash
set -e
set -x

SERVERNAME="$1"
OPENDUT_CERT_CA_NAME="${2:-opendut-ca}"
DEPLOY_DIRECTORY="${3:-deploy}"

PROVISION_ROOT_DIR="/provision/"
OPENDUT_ROOT_CA_NAME="opendut-ca"
PROVISION_PKI_DIR="$PROVISION_ROOT_DIR/pki/"
OPENDUT_PASSWORD_FILE="$PROVISION_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PROVISION_ROOT_DIR/.env"
CA_PATH="$PROVISION_PKI_DIR/$OPENDUT_CERT_CA_NAME"
ROOT_CA_PATH="$PROVISION_PKI_DIR/$OPENDUT_ROOT_CA_NAME"

CERT_PATH="$PROVISION_PKI_DIR/$SERVERNAME"
mkdir -p "$PROVISION_PKI_DIR/$DEPLOY_DIRECTORY"
CERT_DEPLOY_PATH="$PROVISION_PKI_DIR/$DEPLOY_DIRECTORY/$SERVERNAME"

if [ ! -e "$OPENDUT_PASSWORD_FILE" ]; then
  echo "Password file $OPENDUT_PASSWORD_FILE missing. You may override the environment variable OPENDUT_PASSWORD_FILE."
  exit 1
fi

if [ -z "$SERVERNAME" ]; then
  echo "Servername missing"
  echo "$0 <FQDN>"
  exit 1
fi

# certificate signing request
openssl req -new -sha512 -passout file:"$OPENDUT_PASSWORD_FILE" -out "$CERT_PATH".csr -newkey rsa:4096 -keyout "$CERT_PATH".key -subj "/CN=$SERVERNAME/C=XX/ST=Some-State/O=ExampleOrg"


# Create a v3 ext file with SAN properties
cat > "$CERT_PATH".v3.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $SERVERNAME
EOF


# certificate signing
openssl x509 -req -in "$CERT_PATH".csr -CA "$CA_PATH".pem -CAkey "$CA_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -CAcreateserial -outform PEM -out "$CERT_PATH".pem -days 9999 -sha256 -extfile "$CERT_PATH".v3.ext

# Copy certificate
cp "$CERT_PATH".pem "$CERT_DEPLOY_PATH".pem
if [ "$OPENDUT_CERT_CA_NAME" != "$OPENDUT_ROOT_CA_NAME" ]; then
  # append intermediate certificate (certificate chain)
  cat "$CA_PATH".pem >> "$CERT_DEPLOY_PATH".pem
else
  echo "Not appending intermediate certificate."
fi
# Extract private key
openssl rsa -in "$CERT_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -out "$CERT_DEPLOY_PATH".key

rm "$CERT_PATH".csr
rm "$CERT_PATH".v3.ext