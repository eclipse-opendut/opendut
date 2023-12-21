#!/bin/bash
set -e  # exit on error
set -x  # print commands

source "$(dirname "$0")/functions.sh"

copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA1"
copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA2"
append_data_from_env_variable OPENDUT_HOSTS /etc/hosts

ls -la /opt/artifacts

# unpack binaries
tar xf artifacts/opendut-edgar-x86_64-unknown-linux-gnu*
#tar xf artifacts/opendut-cleo-x86_64-unknown-linux-gnu*

# symlink binaries to known binary path
ln -sf /opt/opendut-network/edgar/netbird/netbird /usr/local/sbin/netbird
ln -sf /opt/opendut-edgar/opendut-edgar /usr/local/sbin/opendut-edgar


exec "$@"