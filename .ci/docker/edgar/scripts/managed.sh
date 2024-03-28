#!/bin/bash

source "$(dirname "$0")/functions.sh"

set -e  # exit on error
set -x  # print commands
trap error_stop_and_keep_running TERM
trap error_stop_and_keep_running EXIT

cleo_get_peer_id() {
  edgar_hostname="$1"
  RESULT=$(opendut-cleo list --output json peers  | jq --arg NAME "$edgar_hostname" '.[] | select(.name==$NAME)')
  if [ -n "$RESULT" ]; then
      echo "$RESULT"
  else
      return 1
  fi
}

check_expected_number_of_connected_peers_in_cluster() {
  expected="$1"
  cluster="$2"
  RESULT=$(opendut-cleo list --output json peers | jq --arg CLUSTER "$cluster" -r '. | map(select(.name | contains($CLUSTER))) | .[].status' | grep -c Connected)
  if [ "$RESULT" -eq "$expected" ]; then
      return 0
  else
      return 1
  fi
}

check_interface_exists() {
  interface="$1"

  ip link show dev "$interface" > /dev/null
  EXISTS=$?

  if [ $EXISTS -ne 0 ]; then
    echo "Network interface '$interface' does not exist."
    return 1
  fi
}

pre_flight_tasks() {
  if ! type opendut-cleo > /dev/null; then
    echo "Command 'opendut-cleo' not found."
    exit 1
  fi

  # prepare log directories
  mkdir -p /logs/{opendut-edgar,netbird,journal}

  # opendut-edgar setup.log
  touch /logs/opendut-edgar/setup.log
  if [ ! -e /opt/opendut-edgar/setup.log ]; then
    touch /logs/opendut-edgar/setup.log
    ln -s /logs/opendut-edgar/setup.log /opt/opendut-edgar/setup.log
  fi
  # systemd journal directory
  if [ ! -e /var/log/journal ]; then
    ln -s /logs/journal /var/log/journal
  fi
  # netbird log directory
  if [ ! -e /var/log/netbird ]; then
    ln -s /logs/netbird /var/log/netbird
  fi


  ip link add dev vcan0 type vcan
  ip link add dev vcan1 type vcan
  ip link set dev vcan0 up
  ip link set dev vcan1 up
}

## MAIN TASKS
pre_flight_tasks

PEER_ID=$(uuidgen)
NAME="${OPENDUT_EDGAR_CLUSTER_NAME}_$(hostname)"
echo "Creating peer with name $NAME and id $PEER_ID"
opendut-cleo create peer --name "$NAME" --id "$PEER_ID" --location "$NAME"

DEVICE_INTERFACE="dut0"
ip link add $DEVICE_INTERFACE type dummy
ip link set dev $DEVICE_INTERFACE up
opendut-cleo create network-interface --peer-id "$PEER_ID" --type ethernet --name "$DEVICE_INTERFACE"
opendut-cleo create network-interface --peer-id "$PEER_ID" --type can --name vcan0
opendut-cleo create network-interface --peer-id "$PEER_ID" --type can --name vcan1
opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME" --interface "$DEVICE_INTERFACE" --tag "$OPENDUT_EDGAR_CLUSTER_NAME"
opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME"-vcan0 --interface vcan0 --tag "$OPENDUT_EDGAR_CLUSTER_NAME"
opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME"-vcan1 --interface vcan1 --tag "$OPENDUT_EDGAR_CLUSTER_NAME"

PEER_SETUP_KEY=$(opendut-cleo generate-peer-setup --id "$PEER_ID")
echo "Setting up peer with setup key $PEER_SETUP_KEY"

opendut-edgar setup --no-confirm managed "$PEER_SETUP_KEY"

START_TIME="$(date +%s)"
while ! cleo_get_peer_id "$NAME"; do
    check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for edgar to register"; exit 1; }
    echo "Waiting for edgar to register ..."
    sleep 3
done

expected_peer_count=$((OPENDUT_EDGAR_REPLICAS + 1))
START_TIME="$(date +%s)"
while ! check_expected_number_of_connected_peers_in_cluster "$expected_peer_count" "$OPENDUT_EDGAR_CLUSTER_NAME"; do
  check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for other edgar peers in my cluster."; exit 1; }

  echo "Waiting for all edgar peers in my cluster ..."
  sleep 3
done


if [ "$1" == "leader" ]; then
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


BRIDGE="br-opendut"  # needs to match EDGAR's default
GRE_INTERFACE="gre-opendut0"  # needs to match EDGAR's default prefix

check_edgar_interfaces_exist() {
  check_interface_exists "$BRIDGE"
  check_interface_exists "$GRE_INTERFACE"
}

START_TIME="$(date +%s)"
while ! check_edgar_interfaces_exist; do
    check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for the EDGAR-managed network interfaces to exist."; exit 1; }
    echo "Waiting for the EDGAR-managed network interfaces to exist..."
    sleep 3
done

# Derive the bridge address from the existing address, by replacing '32' with '33'
# eth0 and ip range 192.168.32.0/24 is the main interface of the docker container
# We use 192.168.33.0/24 for the bridge interfaces
BRIDGE_ADDRESS=$(ip -json address show dev eth0 | jq --raw-output '.[0].addr_info[0].local' | sed --expression 's#32#33#')  # derive from existing address, by replacing '32' with '33'
ip address add "$BRIDGE_ADDRESS/24" dev "$BRIDGE"


# create file to signal success to THEO (omitting newline with argument '-n')
echo -n "Success" | tee > /opt/signal/success.txt
echo -n "Success" | tee > /opt/signal/result.txt

trap die_with_success TERM
trap die_with_success EXIT

sleep infinity &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
