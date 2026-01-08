#!/bin/bash
set -e
set -x
# Generate a client certificate signed by the OpenDUT CA for authentication purposes
# Usage: ./generate-client-certificate.sh [client-name]

: '
# Build provision container
docker compose \
      --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml \
      --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/.env.development \
      build provision-secrets
# Run custom provision container where the CA and password file are located
docker compose \
      --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml \
      --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/.env.development \
      run --name=opendut-custom-provision --entrypoint="sleep 3000" --detach --rm provision-secrets
# Run shell in the provision container
docker exec -ti opendut-custom-provision bash
# After exiting the container, stop and remove it
docker stop opendut-custom-provision
'
CLIENT_NAME="${1:-$OPENDUT_CERT_CA_NAME-auth-client}"
OPENDUT_CERT_CA_NAME="${2:-opendut-ca}"
DEPLOY_DIRECTORY="${3:-deploy}"

PROVISION_ROOT_DIR="/provision/"
OPENDUT_ROOT_CA_NAME="opendut-ca"
PROVISION_PKI_DIR="$PROVISION_ROOT_DIR/pki/"
OPENDUT_PASSWORD_FILE="$PROVISION_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PROVISION_ROOT_DIR/.env"
CA_PATH="$PROVISION_PKI_DIR/$OPENDUT_CERT_CA_NAME"
ROOT_CA_PATH="$PROVISION_PKI_DIR/$OPENDUT_ROOT_CA_NAME"

CLIENT_CERT_PATH="$PROVISION_PKI_DIR/$CLIENT_NAME"
mkdir -p "$PROVISION_PKI_DIR/$DEPLOY_DIRECTORY"
CLIENT_DEPLOY_PATH="$PROVISION_PKI_DIR/$DEPLOY_DIRECTORY/$CLIENT_NAME"

# Create certificate signing request (CSR) and private key for the client
openssl req -new -sha512 -passout file:"$OPENDUT_PASSWORD_FILE" -out "$CLIENT_CERT_PATH".csr -newkey rsa:4096 -keyout "$CLIENT_CERT_PATH".key -subj "/CN=$CLIENT_NAME/C=XX/ST=Some-State/O=ExampleOrg"

# Create certificate extension file to define usage for client authentication
cat > "$CLIENT_CERT_PATH".v3.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
extendedKeyUsage = clientAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = $CLIENT_NAME
EOF

# Sign the client CSR with the CA to create the client certificate
SERIAL_FILE="$CA_PATH".srl
if [ -f "$SERIAL_FILE" ]; then
  openssl x509 -req -in "$CLIENT_CERT_PATH".csr -CA "$CA_PATH".pem -CAkey "$CA_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -CAserial "$SERIAL_FILE" -outform PEM -out "$CLIENT_CERT_PATH".pem -days 9999 -sha256 -extfile "$CLIENT_CERT_PATH".v3.ext
else
  openssl x509 -req -in "$CLIENT_CERT_PATH".csr -CA "$CA_PATH".pem -CAkey "$CA_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -CAcreateserial -outform PEM -out "$CLIENT_CERT_PATH".pem -days 9999 -sha256 -extfile "$CLIENT_CERT_PATH".v3.ext
fi

# Store client certificate and key for deployment without password protection
cp "$CLIENT_CERT_PATH".pem "$CLIENT_DEPLOY_PATH".pem
openssl x509 -in "$CLIENT_CERT_PATH".pem -out "$CLIENT_DEPLOY_PATH".crt -outform PEM
if [ "$OPENDUT_CERT_CA_NAME" != "$OPENDUT_ROOT_CA_NAME" ]; then
  # append intermediate certificate (certificate chain)
  cat "$CA_PATH".pem >> "$CLIENT_DEPLOY_PATH".pem
  # verify: pass intermediate CA as untrusted otherwise the chain is not evaluated
  openssl verify -CAfile "$ROOT_CA_PATH".pem -untrusted "$CA_PATH".pem "$CLIENT_DEPLOY_PATH".pem
fi
# Extract private key
openssl rsa -in "$CLIENT_CERT_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -out "$CLIENT_DEPLOY_PATH".key

# Convert to PKCS#12 format for import in clients that require it (e.g., browsers)
openssl pkcs12 -export -in "$CLIENT_CERT_PATH".pem -inkey "$CLIENT_CERT_PATH".key -out "$CLIENT_DEPLOY_PATH".p12 -name "$CLIENT_NAME" -passin file:"$OPENDUT_PASSWORD_FILE" -passout pass:

rm "$CLIENT_CERT_PATH".csr
rm "$CLIENT_CERT_PATH".v3.ext
