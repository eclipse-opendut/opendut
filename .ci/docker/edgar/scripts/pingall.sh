#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

wait_for_peers_to_connect

echo "first ping may take multiple seconds"
IPS=$(netbird status --json | jq -r '.peers.details[].netbirdIp')
for ip in $IPS
do
  ping -c1 "$ip"
done


set -e  # exit on error
set -x  # print each command

echo "-------------------------------------------------------------------------"
echo "Running ping test"
IPS=$(netbird status --json | jq -r '.peers.details[].netbirdIp')
for ip in $IPS
do
  ping -c1 "$ip"
done

echo "SUCCESS"
exit 0
