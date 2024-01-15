#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

## Main
trap die_with_error TERM


wait_for_file_to_be_present "/opt/opendut-network/edgar/netbird/netbird" 600 || { echo "Timeout while waiting for netbird binary to be extracted by edgar"; exit 1; }


wait_for_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
