# CARL gRPC API

Here is an example script showing how use the gRPC API of CARL with authentication and TLS:

```bash
#!/bin/bash

# To use this script, provide a .env file in the working directory with:
#   SSL_CERT_FILE=<PATH>
#   OPENDUT_REPO_ROOT=<PATH>
#   OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET=<SECRET>
# 
# The $OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET can be found in .ci/deploy/localenv/data/secrets/.env under the openDuT repository on the server hosting CARL.
# The CA certificate can be found in .ci/deploy/localenv/data/secrets/pki/opendut-ca.pem. Place this in the $SSL_CERT_FILE path.
# Additionally clone the openDuT repository to your PC into the path $OPENDUT_REPO_ROOT.
# You also have to install the `grpcurl` utility.

set -euo pipefail

set -a  # exports all envs
source .env
set +a

export TOKEN=$(curl \
    --silent \
    --noproxy "*" \
    --data "client_id=opendut-cleo-client" \
    --data "client_secret=${OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET}" \
    --data "grant_type=client_credentials" \
    "https://auth.opendut.local/realms/opendut/protocol/openid-connect/token" \
    | jq --raw-output .access_token
)

grpcurl \
    -proto $OPENDUT_REPO_ROOT/opendut-carl/opendut-carl-api/proto/opendut/carl/services/metadata-provider.proto \
    -import-path $OPENDUT_REPO_ROOT/opendut-carl/opendut-carl-api/proto/ \
    -import-path $OPENDUT_REPO_ROOT/opendut-model/proto/ \
    -expand-headers \
    -H 'Authorization: Bearer ${TOKEN}' \
    carl.opendut.local:443 \
    opendut.carl.services.metadata_provider.MetadataProvider/Version

```
