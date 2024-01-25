#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

ping_all_peers() {
  IPS=$(netbird status --json | jq -r '.peers.details[].netbirdIp')
  for ip in $IPS
  do
    fping -c1 -t500 "$ip" || { echo "$ip did not respond"; sleep 3; }
  done
}


wait_for_peers_to_connect

echo "first ping may take multiple seconds"
ping_all_peers
ping_all_peers

set -e  # exit on error
set -x  # print each command

echo "-------------------------------------------------------------------------"
echo "Running ping test"
ping_all_peers

echo "SUCCESS"
exit 0
