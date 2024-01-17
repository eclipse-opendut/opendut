#!/bin/bash

source "$(dirname "$0")/functions.sh"

set -e  # exit on error
set -x  # print commands
trap die_with_error TERM

cleo_get_peer_id() {
  edgar_hostname="$1"
  RESULT=$(opendut-cleo list --output json peers  | jq --arg NAME "$edgar_hostname" '.[] | select(.name==$NAME)')
  if [ -n "$RESULT" ]; then
      echo "$RESULT"
  else
      return 1
  fi
}

cleo_count_connected_peers() {
  expected="$1"
  RESULT=$(opendut-cleo list --output json peers | jq -r '.[].status' | grep -c Connected)
  if [ "$RESULT" -eq "$expected" ]; then
      return 0
  else
      return 1
  fi
}

cleo_count_connected_peers_in_cluster() {
  expected="$1"
  cluster="$2"
  RESULT=$(opendut-cleo list --output json peers | jq --arg CLUSTER "$cluster" -r '. | map(select(.name | contains($CLUSTER))) | .[].status' | grep -c Connected)
  if [ "$RESULT" -eq "$expected" ]; then
      return 0
  else
      return 1
  fi
}

pre_flight_tasks() {
  touch /etc/security/capability.conf

  if ! type opendut-cleo > /dev/null; then
    echo "Command 'opendut-cleo' not found."
    exit 1
  fi
}

## MAIN TASKS
pre_flight_tasks

PEER_ID=$(uuidgen)
NAME="${OPENDUT_EDGAR_CLUSTER_NAME}_$(hostname)"
echo "Creating peer with name $NAME and id $PEER_ID"
opendut-cleo create peer --name "$NAME" --id "$PEER_ID"
opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME" --interface eth0 --location "$NAME" --tags "$OPENDUT_EDGAR_CLUSTER_NAME"

PEER_SETUP_KEY=$(opendut-cleo generate-peer-setup --id "$PEER_ID" | grep -A1 "Copy the generated setup key" | tail -n1 | sed -e 's#"##g' | sed -e 's/\x1b\[[0-9;]*m//g')
echo "Setting up peer with setup key $PEER_SETUP_KEY"

echo y | opendut-edgar setup managed "$PEER_SETUP_KEY"

START_TIME="$(date +%s)"
while ! cleo_get_peer_id "$NAME"; do
    check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for edgar to register"; exit 1; }
    echo "Waiting for edgar to register ..."
    sleep 3
done

expected_peer_count=$((OPENDUT_EDGAR_REPLICAS + 1))
START_TIME="$(date +%s)"
while ! cleo_count_connected_peers_in_cluster "$expected_peer_count" "$OPENDUT_EDGAR_CLUSTER_NAME"; do
  check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for other edgar peers in my cluster."; exit 1; }

  echo "Waiting for all edgar peers in my cluster ..."
  sleep 3
done

if [ "$1" == "router" ]; then
  DEVICES="$(opendut-cleo list --output=json devices | jq --arg NAME "$OPENDUT_EDGAR_CLUSTER_NAME" -r '.[] | select(.tags==$NAME).name' | xargs echo)"
  echo "Enumerating devices to join cluster: $DEVICES"

  echo "Creating cluster configuration"
  # currently CLEO does not split the string of the device names therefore passing it without quotes
  # shellcheck disable=SC2086
  RESPONSE=$(opendut-cleo create --output=json cluster-configuration --name "$OPENDUT_EDGAR_CLUSTER_NAME" \
      --leader-id "$PEER_ID" \
      --device-names $DEVICES)
  echo "Cluster create result: $RESPONSE"

  CLUSTER_ID=$(echo "$RESPONSE" | jq -r '.id')
  echo "Creating cluster deployment for id=$CLUSTER_ID"
  opendut-cleo create cluster-deployment --id "$CLUSTER_ID"

fi

trap die_with_success TERM

echo "Success"
sleep infinity &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
