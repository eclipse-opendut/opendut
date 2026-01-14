#!/bin/bash

PROVISION_ROOT_DIR="/provision/"
PROVISION_PKI_DIR="$PROVISION_ROOT_DIR/pki/"
OPENDUT_PASSWORD_FILE="$PROVISION_ROOT_DIR/.env-pki"
OPENDUT_ENV_FILE="$PROVISION_ROOT_DIR/.env"
CA_PATH="$PROVISION_PKI_DIR/opendut-ca"

INTERMEDIATE_CA_NAME="${1:-intermediate-ca}"
CERT_PATH="$PROVISION_PKI_DIR/$INTERMEDIATE_CA_NAME"
CERT_DEPLOY_PATH="$PROVISION_PKI_DIR/deploy/$INTERMEDIATE_CA_NAME"



# certificate signing request
openssl req -new -sha512 -passout file:"$OPENDUT_PASSWORD_FILE" -out "$CERT_PATH".csr -newkey rsa:4096 -keyout "$CERT_PATH".key -subj "/CN=$INTERMEDIATE_CA_NAME/C=XX/ST=Some-State/O=ExampleOrg"

# create extension file for intermediate CA (CA:true)
cat > "$CERT_PATH".v3.ext << EOF
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
basicConstraints = critical, CA:true, pathlen:0
keyUsage = critical, digitalSignature, cRLSign, keyCertSign
EOF

# sign intermediate CA
openssl x509 -req -in "$CERT_PATH".csr -CA "$CA_PATH".pem -CAkey "$CA_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -CAcreateserial -outform PEM -out "$CERT_PATH".pem -days 9999 -sha256 -extfile "$CERT_PATH".v3.ext

# copy certificate to deployed path
cp "$CERT_PATH".pem "$CERT_DEPLOY_PATH".crt

# append root CA to create certificate chain
cp "$CERT_PATH".pem "$CERT_DEPLOY_PATH".pem
cat "$CA_PATH".pem >> "$CERT_DEPLOY_PATH".pem
# extract private key
openssl rsa -in "$CERT_PATH".key -passin file:"$OPENDUT_PASSWORD_FILE" -out "$CERT_DEPLOY_PATH".key

rm "$CERT_PATH".csr
rm "$CERT_PATH".v3.ext
