#!/bin/bash

copy_custom_certificate_from_environment_variable() {
  # takes a variable name and pipes the content to a certificate file
  custom_ca_variable_name="$1"
  if [ -n "${!custom_ca_variable_name}" ]; then
    echo -e "${!custom_ca_variable_name}" > /usr/local/share/ca-certificates/opendut_custom_ca_"${custom_ca_variable_name}".crt
    update-ca-certificates
  fi
}
append_data_from_env_variable() {
  var_name="$1"
  file_name="$2"
  if [ -n "${!var_name}" ]; then
    echo -e "${!var_name}" >> "$file_name"
  fi
}

die_with_error() {
        echo "terminating with error"
        exit 1
}
die_with_success() {
        echo "terminating"
        return 0
}

wait_for_peers_to_connect() {
  connected=0
  while [ $connected -lt $OPENDUT_EDGAR_REPLICAS ]; do
    netbird_status=$(/opt/opendut-network/edgar/netbird/netbird status --json 2>/dev/null | jq .peers.connected 2>/dev/null)
    re='^[0-9]+$'
    if [[ $netbird_status =~ $re ]] ; then
      # peer count is a number
      connected=$netbird_status
    fi
    echo "Waiting for peers to connect. Currently connected ${connected} of ${OPENDUT_EDGAR_REPLICAS}"
    sleep 1
  done
}

debug_show_peers_requesting_router_ip() {
  while true; do
    lookups=$(grep router_ip.txt logs.txt | nl | awk '{print $1}' | tail -n1)
    num_lookups=${lookups:-0}
    echo "${num_lookups} of ${OPENDUT_EDGAR_REPLICAS} peers fetched the router_ip address."
    if [ "${num_lookups}" == "${OPENDUT_EDGAR_REPLICAS}" ]; then
      break
    else
      echo "Waiting for peers to fetch router_ip address."
      sleep 1
    fi
  done
}
