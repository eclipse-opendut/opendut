#!/usr/bin/env bash

source "$(dirname "$0")/functions.sh"

ping_all_netbird_peers() {
  REQUIRED_SUCCESS="$1"
  IPS=$(netbird status --json | jq -r '.peers.details[].netbirdIp')

  if [ -z "$IPS" ]; then #abort if no IPs returned
    echo "Failed to determine IP addresses to ping."
    return 1
  fi

  for ip in $IPS
  do
    if [ "$REQUIRED_SUCCESS" == "true" ]; then
      fping --count=1 --timeout=1000 --retry=5 "$ip" || { echo "$ip did not respond"; return 1; }
    else
      fping --count=1 --timeout=1000 --retry=5 "$ip" || { echo "$ip did not respond"; sleep 10; }
    fi
  done
}

ping_all_dut_bridges() {
  REQUIRED_SUCCESS="$1"
  IPS=$(wg show all endpoints | grep -Eo '192.168.32.[0-9]+' | sed -e 's#32#33#')

  if [ -z "$IPS" ]; then #abort if no IPs returned
    echo "Failed to determine IP addresses to ping."
    return 1
  fi

  for ip in $IPS
  do
    if [ "$REQUIRED_SUCCESS" == "true" ]; then
      fping --count=1 --timeout=1000 --retry=5 "$ip" || { echo "$ip did not respond"; return 1; }
    else
      fping --count=1 --timeout=1000 --retry=5 "$ip" || { echo "$ip did not respond"; sleep 10; }
    fi
  done
}


wait_for_peers_to_connect

echo "first ping may take multiple seconds"
ping_all_netbird_peers "false"
ping_all_netbird_peers "false"

set -e  # exit on error
set -x  # print each command

echo "-------------------------------------------------------------------------"
echo "Running ping test"
echo "Pinging NetBird peers..."
ping_all_netbird_peers "true"

echo "Pinging DUT bridges..."
ping_all_dut_bridges "true"

echo "SUCCESS"
exit 0
