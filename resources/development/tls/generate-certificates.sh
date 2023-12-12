#!/bin/sh

set -e

CANAME="insecure-development-ca"

# CA private key
openssl genrsa -out $CANAME.key 4096

# CA public key
openssl req -x509 -new -noenc -key $CANAME.key -sha256 -days 9999 -outform PEM -out $CANAME.pem -subj '/CN=Development CA/C=XX/ST=Some-State/O=Example Org'


SERVERNAME="insecure-development-carl"

# CARL certificate signing request
openssl req -new -noenc -out $SERVERNAME.csr -newkey rsa:4096 -keyout $SERVERNAME.key -subj '/CN=localhost/C=XX/ST=Some-State/O=Example Org'


# create a v3 ext file for SAN properties
cat > $SERVERNAME.v3.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF


# CARL certificate signing
openssl x509 -req -in $SERVERNAME.csr -CA $CANAME.pem -CAkey $CANAME.key -CAcreateserial -outform PEM -out $SERVERNAME.pem -days 9999 -sha256 -extfile $SERVERNAME.v3.ext


rm $SERVERNAME.csr
rm $SERVERNAME.v3.ext
