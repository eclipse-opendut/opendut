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

cleo_check_peer_online() {
  PEER_NAME="$1"
  RESULT=$(opendut-cleo list --output json peers  | jq -r --arg PEER_NAME "$PEER_NAME" '.[] | select(.name==$PEER_NAME).status')
  if [ -n "$RESULT" ]; then
      if [[ "$RESULT" == "Connected" ]]; then
         return 0
      else
        return 1
      fi
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
START_TIME="$(date +%s)"
while ! cleo_check_peer_online "$PEER_NAME"; do
    check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for EDGAR to register."; exit 1; }
    echo "Waiting for EDGAR to register ..."
    sleep 3
done

############################################################
# Wait for other peers to become online
############################################################
expected_peer_count=$((OPENDUT_EDGAR_REPLICAS + 1))
START_TIME="$(date +%s)"
while ! cleo_check_expected_number_of_connected_peers_in_cluster "$expected_peer_count" "test-environment-cluster"; do
  check_timeout "$START_TIME" 600 || { echo "Timeout while waiting for other EDGAR peers in my cluster."; exit 1; }

  echo "Waiting for all EDGAR peers in my cluster ..."
  sleep 3
done


############################################################
# Deploy cluster
############################################################
if [ "$CONTAINER_SERVICE_NAME" == "leader" ]; then
  opendut-cleo apply "/opt/configurations/cluster_configuration.yaml"
  opendut-cleo delete cluster-deployment "206e5d0d-029d-4b03-8789-e0ec46e5a6ba" || echo "i dont mind if it ain't there"
  opendut-cleo create cluster-deployment "206e5d0d-029d-4b03-8789-e0ec46e5a6ba"
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
# TODO: make these hardcoded network ranges configurable/transparent
BRIDGE_ADDRESS=$(ip -json address show dev eth0 | jq --raw-output '.[0].addr_info[0].local' | sed --expression 's#32#33#')  # derive from existing address, by replacing '32' with '33'
ip address add "$BRIDGE_ADDRESS/24" dev "$BRIDGE"
ip address show dev "$BRIDGE"

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
