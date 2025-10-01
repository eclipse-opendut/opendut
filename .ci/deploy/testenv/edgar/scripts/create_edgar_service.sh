#!/bin/bash
set -e

# Simple script to create a peer and immediately start EDGAR as a service (connecting to CARL).
# Can be used in `cargo theo dev edgar-shell`.

source "$(dirname "$0")/functions.sh"
extract_opendut_artifact opendut-edgar "true"
extract_opendut_artifact opendut-cleo "true"

# create peer
PEER_ID="$(uuidgen)"
PEER_NAME="peer-$PEER_ID"

# create interfaces for testing
DEVICE_INTERFACE=dut0
ip link add $DEVICE_INTERFACE type veth peer name dut0local
ip link set dev $DEVICE_INTERFACE up
ip link set dev dut0local up
opendut-cleo create peer --name "$PEER_NAME" --id "$PEER_ID" --location "$PEER_NAME"
opendut-cleo create network-interface --peer-id "$PEER_ID" --type ethernet --name "$DEVICE_INTERFACE"
opendut-cleo create device --peer-id "$PEER_ID" --name "$PEER_NAME-device-$DEVICE_INTERFACE" --interface "$DEVICE_INTERFACE"

# setup peer
PEER_SETUP_STRING=$(opendut-cleo generate-setup-string "$PEER_ID")
opendut-edgar setup managed --no-confirm "$PEER_SETUP_STRING"
