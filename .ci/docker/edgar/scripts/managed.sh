#!/bin/bash

source "$(dirname "$0")/functions.sh"

set -e  # exit on error
set -x  # print commands
trap error_stop_and_keep_running TERM
trap error_stop_and_keep_running EXIT

cleo_get_peer_id() {
  PEER_NAME="$1"
  RESULT=$(opendut-cleo list --output json peers  | jq -r --arg PEER_NAME "$PEER_NAME" '.[] | select(.name==$PEER_NAME).id')
  if [ -n "$RESULT" ]; then
      echo "$RESULT"
  else
      return 1
  fi
}

cleo_check_expected_number_of_connected_peers_in_cluster() {
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

check_gre_interface_exists() {
  RESULT=$(ip -json l show type gretap | jq -r '.[] | select(.altnames != null) | select(.altnames[0] | startswith("opendut"))')

  if [ -z "$RESULT" ]; then
    echo "GRE interface does not yet exist!"
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

  # create dummy ethernet device
  ip link add dut0 type dummy
  ip link set dev dut0 up

  # add second dummy interface for testing
  ip link add dut1 type dummy
  ip link set dev dut1 up

  # create dummy can devices
  ip link add dev vcan0 type vcan
  ip link add dev vcan1 type vcan
  ip link set dev vcan0 up
  ip link set dev vcan1 up

  if [ "$1" == "leader" ]; then
    /usr/local/bin/start-docker.sh
    echo "Building Docker image for Nmap test"
    docker build --tag "nmap-test" /opt/test_execution_container
  fi
}

pre_flight_tasks "$1"

# Determine docker service name and respective role for EDGAR
CONTAINER_IP="$(ip -4 addr show eth0 | grep -oP "(?<=inet ).*(?=/)")"
CONTAINER_SERVICE_NAME="$(dig -x "${CONTAINER_IP}" +short | grep -Eo "edgar-[a-z0-9\-]+" | cut -d'-' -f'2-')"

# Apply peer configuration
opendut-cleo apply "/opt/configurations/peer_descriptor_${CONTAINER_SERVICE_NAME}.yaml"
PEER_NAME="test-environment-cluster-${CONTAINER_SERVICE_NAME}"
PEER_ID=$(cleo_get_peer_id "$PEER_NAME")

# Setup EDGAR
PEER_SETUP_STRING=$(opendut-cleo generate-setup-string "$PEER_ID")
echo "Setting up peer with Setup-String: $PEER_SETUP_STRING"
opendut-edgar setup --no-confirm managed "$PEER_SETUP_STRING"


############################################################
# Wait for edgar to be declared online/connected
############################################################
opendut-cleo await peer-online "$PEER_ID"

############################################################
# Wait for other peers to be online
############################################################
opendut-cleo await peer-online --peers-may-not-yet-exist \
  "bcf75b6c-d6e1-42bd-b74e-30690bca88ab" \
  "d629fede-27c8-4270-8e73-f91ae7d31a33" \
  "525b369f-8abb-4b49-8046-25948936ad6c" \
  "8b5835af-0e3c-4a28-a7d7-623a929a0f1b" \
  "a1db14f5-1d08-4876-adf2-ba32d99f25ff"


############################################################
# Deploy cluster
############################################################
if [ "$CONTAINER_SERVICE_NAME" == "leader" ]; then
  opendut-cleo apply "/opt/configurations/cluster_configuration.yaml"
  opendut-cleo delete cluster-deployment "206e5d0d-029d-4b03-8789-e0ec46e5a6ba" || echo "i dont mind if it ain't there"
  opendut-cleo create cluster-deployment "206e5d0d-029d-4b03-8789-e0ec46e5a6ba"

  # Wait for cluster peers to be online
  opendut-cleo await cluster-peers-online "206e5d0d-029d-4b03-8789-e0ec46e5a6ba"
fi


BRIDGE="br-opendut"  # needs to match EDGAR's default

check_edgar_interfaces_exist() {
  check_interface_exists "$BRIDGE"
  check_gre_interface_exists
}

START_TIME="$(date +%s)"
while ! check_edgar_interfaces_exist; do
    check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for the EDGAR-managed network interfaces to exist."; exit 1; }
    echo "Waiting for the EDGAR-managed network interfaces to exist..."
    sleep 3
done

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
