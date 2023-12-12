#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

## Main
trap die_with_error TERM

until [ -f /opt/opendut-network/edgar/netbird/netbird ]
do
  echo "Waiting for netbird to be provisioned ..."
  sleep 1
done

wait_for_peers_to_connect
