#!/bin/bash

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
      return 1
    fi
    if curl --silent "$url" --output /dev/null; then
      break
    fi
    echo "Waiting for $url to be available..."
    sleep "$sleep_time"
  done
}

wait_for_netbird_user_to_be_synced_from_keycloak() {
  local user_name="$1"
  local timeout="${2:-60}"
  local sleep_time="${3:-5}"
  local start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    local now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout ($timeout seconds) while waiting for $user_name to be synced from keycloak"
      return 1
    fi
    if user_present "$user_name"; then
      break
    fi
    echo "Waiting for user $user_name to become available..."
    sleep "$sleep_time"
  done
}

users_list() {
  USERS=$(curl --silent -H "Authorization: Bearer $TOKEN" "$NETBIRD_MANAGEMENT_URL"/api/users)
  # shellcheck disable=SC2181
  if [[ -z "$USERS" || $? -ne 0 ]]; then
    echo ""
    return 1
  fi
  echo "$USERS"
}

get_user_oauth_token() {
  # requires public client and client with password grant enabled, directAccessGrantsEnabled=true
  RESPONSE=$(curl -s -d "client_id=netbird-mgmt-cli" -d "username=netbird" -d "password=$NETBIRD_PASSWORD" -d "grant_type=password" "$KEYCLOAK_URL"/realms/netbird/protocol/openid-connect/token)
  TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')
  echo "$TOKEN"
}

netbird_auth() {
  TOKEN=$(get_user_oauth_token)
  export TOKEN
}

user_present() {
  USER_NAME="$1"

  netbird_auth
  USERS=$(users_list)
  if [ -n "$USERS" ]; then
    NETBIRD_USER_ID=$(echo "$USERS" | jq -r ".[] | select(.name==\"$USER_NAME\").id" 2>/dev/null)
    if [ -z "$NETBIRD_USER_ID" ]; then
      echo "$USERS"
      echo "NetBird user $USER_NAME is not present"
      return 1
    else
      echo "NetBird user $USER_NAME is present"
      return 0
    fi
  else
    echo "NetBird user $USER_NAME is not present"
    return 1
  fi
}
get_netbird_api_token() {
  USERS=$(users_list)
  NETBIRD_USER_ID=$(echo "$USERS" | jq -r '.[] | select(.name=="netbird").id')

  API_KEY_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
       -H 'Content-Type application/json' \
       -d "{\"user_id\": \"$NETBIRD_USER_ID\", \"name\": \"admin\", \"expires_in\": 365 }" \
       "$NETBIRD_MANAGEMENT_URL"/api/users/"$NETBIRD_USER_ID"/tokens)

  API_KEY=$(echo "$API_KEY_RESPONSE" | jq -r '.plain_token')
  echo "$API_KEY"
}
