#!/bin/bash

# set ip address of bridge interface for ping testing

DUT_NAME="dut0local"  # needs to match configured device in managed.sh
DUT_ADDRESS=$(ip -json address show dev eth0 | jq --raw-output '.[0].addr_info[0].local' | sed --expression 's#32#33#')  # derive from existing address, by replacing '32' with '33'
ip address add "$DUT_ADDRESS/24" dev "$DUT_NAME"
ip address show dev "$DUT_NAME"
