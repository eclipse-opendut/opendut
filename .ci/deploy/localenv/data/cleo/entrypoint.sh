#!/bin/bash

cd /usr/local/bin || { echo "Failed to change directory"; exit; }

wait_for_url() {
  local url="$1"
  shift 1
  # remaining arguments are curl arguments
  local curl_args=( "$@" )
  local timeout=600
  local sleep_time=5

  start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout while waiting for $url"
      return 1
    fi
    if curl --silent --output /dev/null -w "Waiting response code: %{http_code} URL: $url\n" "$url" "${curl_args[@]}"; then
      break
    fi
    echo "Waiting for $url to be available..."
    sleep "$sleep_time"
  done
}

# Download CLEO
CARL_URL="https://${OPENDUT_CLEO_NETWORK_CARL_HOST}"
CLEO_DOWNLOAD_URL="https://${OPENDUT_CLEO_NETWORK_CARL_HOST}/api/cleo/x86_64-unknown-linux-gnu/download"
CURL_ARGS=()

if [ "$OPENDUT_CLEO_NETWORK_TLS_CLIENT_AUTH_ENABLED" == "true" ]; then
  if [ -e "$OPENDUT_CLEO_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE" ] && [ -e "$OPENDUT_CLEO_NETWORK_TLS_CLIENT_AUTH_KEY" ]; then
    CURL_ARGS=("--cert" "$OPENDUT_CLEO_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE" "--key" "$OPENDUT_CLEO_NETWORK_TLS_CLIENT_AUTH_KEY")
  else
    echo "Client TLS authentication is enabled, but certificate or key file does not exist."
    exit 1
  fi
fi

wait_for_url "${CARL_URL}" "${CURL_ARGS[@]}" || { echo "CARL did not respond."; exit 1; }
wait_for_url "${CLEO_DOWNLOAD_URL}" "${CURL_ARGS[@]}" || { echo "CARL did not respond."; exit 1; }

curl "${CURL_ARGS[@]}" "$CLEO_DOWNLOAD_URL" -w "Download response code: %{http_code} URL: $CLEO_DOWNLOAD_URL\n" --output cleo.tar.gz
tar --strip-components=1 -xvf cleo.tar.gz

echo "Keep container running with sleep infinity"
sleep infinity