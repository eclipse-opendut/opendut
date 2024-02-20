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

error_stop_and_keep_running() {
        echo "ERROR occurred. Keeping container running for debugging."
        sleep infinity
}

die_with_success() {
        echo "terminating"
        return 0
}

wait_for_peers_to_connect() {
  local expected_peer_count="$1"
  local timeout="${2:-600}"
  local sleep_time="${3:-5}"

  START_TIME="$(date +%s)"
  END_TIME=$((START_TIME + timeout))

  connected=0
  while [[ "$connected" -lt "$expected_peer_count" ]]; do
    local now
    now=$(date +%s)
    if [ "$now" -gt "$END_TIME" ]; then
      echo "Timeout while waiting for binary to be extracted by edgar: $file"
      return 1
    fi

    netbird_status=$(/opt/opendut/edgar/netbird/netbird status --json 2>/dev/null | jq .peers.connected 2>/dev/null)
    re='^[0-9]+$'
    if [[ $netbird_status =~ $re ]] ; then
      # peer count is a number
      connected=$netbird_status
    fi
    echo "Waiting for peers to connect. Currently connected ${connected} of ${expected_peer_count}"
    sleep "$sleep_time"
  done
}

wait_for_file_to_be_present() {
  local file="$1"
  local timeout="${2:-600}"
  local sleep_time="${3:-5}"

  START_TIME="$(date +%s)"
  END_TIME=$((START_TIME + timeout))
  until [ -f "$file" ];
  do
    local now
    now=$(date +%s)
    if [ "$now" -gt "$END_TIME" ]; then
      echo "Timeout while waiting for binary to be extracted by edgar: $file"
      return 1
    fi
    echo "Waiting for binary to be extracted by edgar: $file"
    sleep "$sleep_time"
  done
}

check_timeout() {
  start_time="$1"
  timeout="$2"

  END_TIME=$((start_time + timeout))
  local now
  now=$(date +%s)
  if [ "$now" -gt "$END_TIME" ]; then
    return 1
  else
    return 0
  fi
}


debug_show_peers_requesting_leader_ip() {
  while true; do
    lookups=$(grep leader_ip.txt logs.txt | nl | awk '{print $1}' | tail -n1)
    num_lookups=${lookups:-0}
    echo "${num_lookups} of ${OPENDUT_EDGAR_REPLICAS} peers fetched the leader_ip address."
    if [ "${num_lookups}" == "${OPENDUT_EDGAR_REPLICAS}" ]; then
      break
    else
      echo "Waiting for peers to fetch leader_ip address."
      sleep 1
    fi
  done
}


extract_opendut_artifact() {
  NAME="$1"
  REQUIRED="${2:-true}"

  if [ "$REQUIRED" = false ] ; then
    echo "Skipping extraction of distribution archive for \'$NAME\'."
  else
    echo "Extracting distribution archive for \'$NAME\'."
    OPENDUT_DIST_ARCHIVE_PATH=$(find "$OPENDUT_ARTIFACTS_DIR" -name "$NAME-x86_64-unknown-linux-gnu*.tar.gz" -print0 | tr '\0' '\n' | head)
    if [ -e "$OPENDUT_DIST_ARCHIVE_PATH" ]; then
      tar xf "$OPENDUT_DIST_ARCHIVE_PATH"
      OPENDUT_ARTIFACT_BINARY="/opt/$NAME/$NAME"
      if [ -e "$OPENDUT_ARTIFACT_BINARY" ]; then
        ln -sf /opt/"$NAME"/"$NAME" /usr/local/opendut/bin/distribution/"$NAME"
      else
        echo "Could not extract artifact binary \'$OPENDUT_ARTIFACT_BINARY\' from archive \'$OPENDUT_DIST_ARCHIVE_PATH\'."
        exit 1
      fi
    else
      echo "Could not find distribution archive for \'$NAME\'."
      if [ "$REQUIRED" = true ] ; then
        exit 1
      fi
    fi
  fi


}
