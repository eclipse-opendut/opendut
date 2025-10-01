#!/usr/bin/env bash

set -x
source "$(dirname "$0")/functions.sh"

ping_all_netbird_peers() {
  REQUIRED_SUCCESS="$1"
  IPS=$(netbird status --json | jq -r '.peers.details[] | select(.status=="Connected") | .netbirdIp')

  if [ -z "$IPS" ]; then #abort if no IPs returned
    echo "Failed to determine IP addresses to ping."
    return 1
  fi

  for ip in $IPS
  do
    if [ "$REQUIRED_SUCCESS" == "true" ]; then
      fping --count=1 --timeout=1000 --retry=20 "$ip" || { echo "$ip did not respond"; return 1; }
    else
      fping --count=1 --timeout=1000 --retry=20 "$ip" || { echo "$ip did not respond"; sleep 10; }
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
      fping --count=1 --timeout=1000 --retry=20 "$ip" || { echo "$ip did not respond"; return 1; }
    else
      fping --count=1 --timeout=1000 --retry=20 "$ip" || { echo "$ip did not respond"; sleep 10; }
    fi
  done
}


wait_for_server_ready() {
  HOST="$1"
  PORT="$2"

  retries=10
  retry_seconds=10

  while [ $retries -gt 0 ]
  do
    nc -z "$HOST" "$PORT"  # probe port without keeping connection

    if [ $? -eq 0 ]; then
      echo "Connection to server $HOST on port $PORT was successful. Server is considered ready."
	    return 0
    fi

    echo "Server $HOST did not respond on port $PORT. Retrying in $retry_seconds seconds..."
    sleep $retry_seconds
    ((retries--))
  done

  return 1
}

#wait_for_server_ready opendut.local 3478 # TODO: localenv netbird-coturn is reachable by this, will be deleted after netbird upgrade
wait_for_server_ready netbird-signal 80
wait_for_server_ready netbird-management 443  # NetBird-API is also in this service


wait_for_netbird_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
wait_for_wireguard_peers_to_connect "$OPENDUT_EDGAR_REPLICAS"
wait_for_edgar_to_create_gre_interfaces "$OPENDUT_EDGAR_REPLICAS"

echo "first ping may take multiple seconds"
ping_all_netbird_peers "false"
ping_all_netbird_peers "false"

set -e  # exit on error
set -x  # print each command

echo "-------------------------------------------------------------------------"
echo " STAGE 1: Running ping test"
echo "          Pinging NetBird peers..."
ping_all_netbird_peers "true"

echo "-------------------------------------------------------------------------"
echo " STAGE 2: Pinging DUT bridges..."
ping_all_dut_bridges "true"

echo "SUCCESS"
exit 0
