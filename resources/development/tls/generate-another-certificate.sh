#/bin/bash
set -e
set -x
CANAME="insecure-development-ca"
SERVERNAME="$1"

if [ -z "$SERVERNAME" ]; then
  echo "Servername missing"
  echo "$0 <FQDN>"
  exit 1
fi

# certificate signing request
openssl req -new -noenc -out $SERVERNAME.csr -newkey rsa:4096 -keyout $SERVERNAME.key -subj "/CN=$SERVERNAME/C=XX/ST=Some-State/O=ExampleOrg"


# create a v3 ext file for SAN properties
cat > $SERVERNAME.v3.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $SERVERNAME
EOF


# CARL certificate signing
openssl x509 -req -in $SERVERNAME.csr -CA $CANAME.pem -CAkey $CANAME.key -CAcreateserial -outform PEM -out $SERVERNAME.pem -days 9999 -sha256 -extfile $SERVERNAME.v3.ext


rm $SERVERNAME.csr
rm $SERVERNAME.v3.ext


