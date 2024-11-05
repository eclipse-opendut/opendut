#!/bin/bash
set -e
set -x
CA_NAME="insecure-development-ca"
SERVERNAME="$1"

if [ -z "$SERVERNAME" ]; then
  echo "Servername missing"
  echo "$0 <FQDN>"
  exit 1
fi

# Generate UUID for the certificate
UUID=$(uuidgen)

# certificate signing request
openssl req -new -noenc -out "$SERVERNAME".csr -newkey rsa:4096 -keyout "$SERVERNAME".key -subj "/CN=$SERVERNAME-$UUID/C=XX/ST=Some-State/O=ExampleOrg"


# create a v3 ext file for SAN properties
cat > "$SERVERNAME".v3.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $SERVERNAME
EOF

# add the alternative name if provided
# user for e.g. netbird
# ./generate-another-certificate.sh netbird-management DNS.2:netbird-dashboard IP.1:192.168.56.10 IP.2:192.168.32.211 IP.3:192.168.32.213
position=0
for var in "$@"
do
  position=$((position+1))
  if [ "$position" -eq "1" ]; then
    continue
  fi
  echo "Assigning alt_name: $var"
  key=$(echo "$var" | cut -d: -f1)
  value=$(echo "$var" | cut -d: -f2)

  echo "$key = $value" >> "$SERVERNAME".v3.ext

done

# CARL certificate signing
openssl x509 -req -in "$SERVERNAME".csr -CA $CA_NAME.pem -CAkey $CA_NAME.key -CAcreateserial -outform PEM -out "$SERVERNAME".pem -days 9999 -sha256 -extfile "$SERVERNAME".v3.ext


rm "$SERVERNAME".csr
rm "$SERVERNAME".v3.ext

