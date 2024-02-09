#!/usr/bin/env bash

set -e

# Add the root certificates to the trusted certificates
# rename all .pem files to .crt

for certificate in /opendut-tls/*-ca.pem; do
  cert_base_name=$(basename "$certificate")
  cert_new_name=${cert_base_name//.pem/.crt}
  echo "Adding $certificate to the trusted certificates"
  cp "$certificate" /usr/local/share/ca-certificates/"$cert_new_name"
done

update-ca-certificates
