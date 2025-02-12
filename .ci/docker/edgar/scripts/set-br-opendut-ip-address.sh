#!/bin/bash

# set ip address of bridge interface for ping testing

BRIDGE="br-opendut"  # needs to match EDGAR's default
BRIDGE_ADDRESS=$(ip -json address show dev eth0 | jq --raw-output '.[0].addr_info[0].local' | sed --expression 's#32#33#')  # derive from existing address, by replacing '32' with '33'
ip address add "$BRIDGE_ADDRESS/24" dev "$BRIDGE"
ip address show dev "$BRIDGE"
