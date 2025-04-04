#!/bin/bash
set -e

# simple script to create a peer and immediately start edgar as a service (connecting to CARL)

source "$(dirname "$0")/functions.sh"
extract_opendut_artifact opendut-edgar "true"
extract_opendut_artifact opendut-cleo "true"

PEER_ID="$(uuidgen)"
PEER_NAME="peer-$PEER_ID"

opendut-cleo create peer --name "$PEER_NAME" --id "$PEER_ID" --location "$PEER_NAME"
PEER_SETUP_STRING=$(opendut-cleo generate-setup-string "$PEER_ID")
opendut-edgar setup --no-confirm managed "$PEER_SETUP_STRING"
