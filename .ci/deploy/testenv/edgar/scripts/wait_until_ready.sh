#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

## Main
trap die_with_error TERM


wait_for_file_to_be_present "/opt/opendut/edgar/netbird/netbird" 600 || { echo "Timeout while waiting for netbird binary to be extracted by edgar"; exit 1; }


role="${1:-leader}"

if [ "$role" == "leader" ]; then
  echo "Waiting for netbird peers to connect."
  echo "Leader will wait for $OPENDUT_EDGAR_REPLICAS peers to connect."
  wait_for_netbird_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
  wait_for_wireguard_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
  wait_for_edgar_to_create_gre_interfaces "$OPENDUT_EDGAR_REPLICAS"
else
  echo "Waiting for netbird peers to connect."
  echo "Peer will wait for  $OPENDUT_EDGAR_REPLICAS peers and 1 GRE interface."
  wait_for_netbird_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
  wait_for_wireguard_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
  wait_for_edgar_to_create_gre_interfaces 1
fi
