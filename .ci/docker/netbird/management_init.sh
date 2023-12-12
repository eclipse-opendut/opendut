#!/bin/bash

ls -la /usr/local/share/ca-certificates/
/usr/sbin/update-ca-certificates

wait_for_url() {
  local url="$1"
  local timeout="${2:-60}"
  local sleep_time="${3:-5}"
  local start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    local now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout while waiting for $url"
      exit 1
    fi
    if curl -sf "$url" --output /dev/null; then
      break
    fi
    echo "Waiting for $url to be available..."
    sleep "$sleep_time"
  done
}

wait_for_url "$KEYCLOAK_URL" 600 5 || exit 1
echo "Keycloak ready"


/go/bin/netbird-mgmt management --port 80 --log-file console --disable-anonymous-metrics=false --single-account-mode-domain=netbird.opendut.local --dns-domain=netbird.opendut.local
