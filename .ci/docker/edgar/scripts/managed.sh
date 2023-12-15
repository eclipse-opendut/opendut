#!/bin/bash

if [ -n "$1" ]; then
  ROUTER=true
else
  ROUTER=false
fi

source "$(dirname "$0")/functions.sh"

copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA1"
copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA2"
append_data_from_env_variable OPENDUT_HOSTS /etc/hosts

ls -la /opt/artifacts

# unpack binaries
tar xf artifacts/opendut-edgar-x86_64-unknown-linux-gnu*
#tar xf artifacts/opendut-cleo-x86_64-unknown-linux-gnu*

# symlink binaries to known binary path
ln -s /opt/opendut-network/edgar/netbird/netbird /usr/local/sbin/netbird
ln -s /opt/opendut-edgar/opendut-edgar /usr/local/sbin/opendut-edgar

touch /etc/security/capability.conf


PEER_ID=$(uuid)
opendut-cleo create peer --name router --id "$PEER_ID"
PEER_SETUP_KEY=$(opendut-cleo generate-peer-setup --id "$PEER_ID" | grep -A1 "Copy the generated setup key" | tail -n1 | sed -e 's#"##g' | sed -e 's/\x1b\[[0-9;]*m//g')
echo y | opendut-edgar setup managed "$PEER_SETUP_KEY"
