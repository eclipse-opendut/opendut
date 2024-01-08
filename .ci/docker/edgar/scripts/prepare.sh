#!/bin/bash

source "$(dirname "$0")/functions.sh"
trap die_with_error TERM


if [ -n "$1" ] ; then
  echo y | /opt/opendut-edgar/opendut-edgar setup unmanaged --setup-key "$NETBIRD_SETUP_KEY" --management-url "${NETBIRD_MANAGEMENT_API}" --router=local

  while ! netbird status | grep IP; do
    echo "Waiting for netbird to start up..."
    sleep 1
  done
  router_ip=$(netbird status | grep IP | grep -Eo "[0-9]+.[0-9]+.[0-9]+.[0-9]+")

  echo "$router_ip" > router_ip.txt
  python3 -m http.server 2> logs.txt &
  python3 ip_provider.py 2> provider.txt &

  debug_show_peers_requesting_router_ip
  wait_for_peers_to_connect

  echo y | /opt/opendut-edgar/opendut-edgar setup unmanaged --setup-key "$NETBIRD_SETUP_KEY" --management-url "${NETBIRD_MANAGEMENT_API}" --router=local
  echo setting bridge ip
  ip a a 192.168.100.1/24 dev br-opendut

else
  echo waiting for router to come up
  while ! curl -sf "http://edgar_router:8000" --output /dev/null; do
    echo "Waiting for router to start up..."
    sleep 3
  done

  router_ip=$(curl --silent http://edgar_router:8000/router_ip.txt)
  echo "Using router router_ip address $router_ip"
  echo y | /opt/opendut-edgar/opendut-edgar setup unmanaged --setup-key "$NETBIRD_SETUP_KEY" --management-url "${NETBIRD_MANAGEMENT_API}" --router="$router_ip"

  echo fetching bridge_ip
  bridge_ip=$(curl --silent http://edgar_router:5000/)
  bridge_ip="192.168.100.${bridge_ip}/24"
  echo "Got bridge ip ${bridge_ip}"
  ip a a "${bridge_ip}" dev br-opendut

fi

trap die_with_success TERM

echo "sleeping"
sleep infinity &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
