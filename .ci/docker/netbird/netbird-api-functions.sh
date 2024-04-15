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


############################################################################################
# KEYCLOAK ADMIN TASKS
get_admin_oauth_token() {
  # requires public client and client with password grant enabled, directAccessGrantsEnabled=true
  RESPONSE=$(curl -s -d "client_id=admin-cli" -d "username=$KEYCLOAK_ADMIN" -d "password=$KEYCLOAK_ADMIN_PASSWORD" -d "grant_type=password" $KEYCLOAK_URL/realms/master/protocol/openid-connect/token)
  ADMIN_TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')
  echo "$ADMIN_TOKEN"
}

keycloak_admin_auth() {
  # ignore "Declare and assign separately to avoid masking return values."
  # shellcheck disable=SC2155
  export ADMIN_TOKEN=$(get_admin_oauth_token)
}

keycloak_list_clients_in_realm_netbird() {
  CLIENTS=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_URL/admin/realms/netbird/clients?first=0&max=11")
  echo "$CLIENTS"
}

keycloak_client_in_realm_netbird_is_present() {
  CLIENT_ID="$1"

  keycloak_admin_auth
  CLIENTS=$(keycloak_list_clients_in_realm_netbird)
  if [ -n "$CLIENTS" ]; then
    KEYCLOAK_CLIENT=$(echo "$CLIENTS" | jq -r ".[] | select(.clientId==\"$CLIENT_ID\")" 2>/dev/null)
    if [ -z "$KEYCLOAK_CLIENT" ]; then
      echo "Keycloak client \'$CLIENT_ID\' is not present"
      return 1
    else
      echo "Keycloak client \'$CLIENT_ID\' is present"
      return 0
    fi
  else
    echo "Keycloak client \'$CLIENT_ID\' is not present"
    return 1
  fi
}

keycloak_user_count() {
  KEYCLOAK_REALM="${1:-netbird}"

  keycloak_admin_auth
  KEYCLOAK_USER_COUNT=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_URL/admin/realms/${KEYCLOAK_REALM}/users/count")
  echo "$KEYCLOAK_USER_COUNT"
}

keycloak_user_list() {
  KEYCLOAK_REALM="${1:-netbird}"

  keycloak_admin_auth
  KEYCLOAK_USERS=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_URL/admin/realms/${KEYCLOAK_REALM}/users")
  echo "$KEYCLOAK_USERS"
}

keycloak_user_find() {
  KEYCLOAK_REALM="${1:-netbird}"
  KEYCLOAK_USER_NAME="${2:-netbird}"

  KEYCLOAK_USER_LIST=$(keycloak_user_list "${KEYCLOAK_REALM}")
  KEYCLOAK_USER_OBJ=$(echo "$KEYCLOAK_USER_LIST" | jq -r ".[] | select(.username==\"$KEYCLOAK_USER_NAME\")" 2>/dev/null)

  echo "$KEYCLOAK_USER_OBJ"
}

keycloak_user_present() {
  KEYCLOAK_REALM="${1:-netbird}"
  KEYCLOAK_USER_NAME="${2:-netbird}"

  KEYCLOAK_USER_OBJ=$(keycloak_user_find "${KEYCLOAK_REALM}" "${KEYCLOAK_USER_NAME}")
  if [ -n "$KEYCLOAK_USER_OBJ" ]; then
    echo "Keycloak user $KEYCLOAK_USER_NAME is present in realm $KEYCLOAK_REALM"
    return 0
  else
    echo "Keycloak user $KEYCLOAK_USER_NAME is not yet available in realm $KEYCLOAK_REALM!"
    return 1
  fi
}

wait_for_keycloak_user__in_realm_netbird() {
  local user_name="${1:-netbird}"
  local timeout="${2:-60}"
  local sleep_time="${3:-5}"
  local start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    local now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout ($timeout seconds) while waiting for netbird client in keycloak"
      return 1
    fi
    keycloak_user_present "netbird" "$user_name" && break
    echo "Waiting for user $user_name to be available in keycloak realm 'netbird' ..."
    sleep "$sleep_time"
  done
}

wait_for_keycloak_client__in_realm_netbird() {
  local user_name="${1:-netbird-backend}"
  local timeout="${2:-60}"
  local sleep_time="${3:-5}"
  local start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    local now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout ($timeout seconds) while waiting for netbird client in keycloak"
      return 1
    fi
    keycloak_client_in_realm_netbird_is_present "$user_name" && break
    echo "Waiting for $user_name to be available..."
    sleep "$sleep_time"
  done
}
############################################################################################

get_user_oauth_token() {
  # requires public client and client with password grant enabled, directAccessGrantsEnabled=true
  RESPONSE=$(curl -s -d "client_id=netbird-mgmt-cli" -d "username=netbird" -d "password=$NETBIRD_PASSWORD" -d "grant_type=password" $KEYCLOAK_URL/realms/netbird/protocol/openid-connect/token)
  TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')
  echo "$TOKEN"
}

get_client_oauth_token() {
    RESPONSE=$(curl -s -d "client_id=netbird-mgmt-cli" -d client_secret="5185e4ca-9436-11ee-b56d-2701aec9048e" -d "grant_type=client_credentials" $KEYCLOAK_URL/realms/netbird/protocol/openid-connect/token)
    TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')
    echo "$TOKEN"
}

netbird_auth() {
  # ignore "Declare and assign separately to avoid masking return values."
  # shellcheck disable=SC2155
  export TOKEN=$(get_user_oauth_token)
}

wait_for_keycloak_client_auth_successful() {
  local timeout="${1:-60}"
  local sleep_time="${2:-5}"
  local start_time=$(date +%s)
  local end_time=$((start_time + timeout))
  while true; do
    local now=$(date +%s)
    if [ "$now" -gt "$end_time" ]; then
      echo "Timeout ($timeout seconds) while waiting for netbird management client to be authenticated"
      return 1
    fi
    netbird_auth
    if [ -n "$TOKEN" ]; then
      echo "Netbird management client 'netbird-mgmt-cli' is authenticated."
      break
    fi
    echo "Waiting for authentication to be available... sleeping $sleep_time seconds"
    sleep "$sleep_time"
  done
}

netbird_api_token_test() {
  TOKEN="${1}"
  RESULT=$(curl --fail --silent -H "Authorization: Token $TOKEN" $NETBIRD_MANAGEMENT_URL/api/groups)
  # shellcheck disable=SC2181
  if [ $? -ne 0 ]; then
    echo "NetBird API token is not valid. Failed to retrieve groups: $RESULT"
    curl --fail -H "Authorization: Token $TOKEN" $NETBIRD_MANAGEMENT_URL/api/groups
    return 1
  fi
  if [ -z "$RESULT" ]; then
    echo "NetBird API token is not valid. Failed to retrieve groups. Result is empty"
    return 1
  fi
  echo "NetBird API token is valid"
}

group_list() {
    GROUP_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" $NETBIRD_MANAGEMENT_URL/api/groups)
    echo "$GROUP_RESPONSE"
}

netbird_online() {
  netbird_auth
  RESPONSE=$(group_list)
  if [ -z "$RESPONSE" ]; then
    echo "NetBird is offline"
    return 1
  else
    echo "NetBird is online"
    return 0
  fi
}

users_list() {
  USERS=$(curl --silent -H "Authorization: Bearer $TOKEN" $NETBIRD_MANAGEMENT_URL/api/users)
  # shellcheck disable=SC2181
  if [[ -z "$USERS" || $? -ne 0 ]]; then
    echo ""
    return 1
  fi
  echo "$USERS"
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


get_netbird_api_token() {
  USERS=$(users_list)
  NETBIRD_USER_ID=$(echo "$USERS" | jq -r '.[] | select(.name=="netbird").id')
  API_KEY_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
       -H 'Content-Type application/json' \
       -d "{\"user_id\": \"$NETBIRD_USER_ID\", \"name\": \"admin\", \"expires_in\": 365 }" \
       $NETBIRD_MANAGEMENT_URL/api/users/"$NETBIRD_USER_ID"/tokens)

  API_KEY=$(echo "$API_KEY_RESPONSE" | jq -r '.plain_token')
  echo "$API_KEY"
}

group_id_get_by_name() {
    GROUP_NAME="$1"

    GROUP_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" $NETBIRD_MANAGEMENT_URL/api/groups)
    GROUP_ID=$(echo "$GROUP_RESPONSE" | jq -sr ".[] | map(select(.name==\"${GROUP_NAME}\")) | first | .id" )
    if [ "$GROUP_ID" == "null" ]; then
      echo ""
    else
      echo "$GROUP_ID"
    fi
}

group_create() {
  GROUP_NAME="$1"
  GROUP_ID=$(group_id_get_by_name "$GROUP_NAME")

  if [ -z "$GROUP_ID" ]; then
      GROUP_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" -d "{\"name\":\"$GROUP_NAME\"}" $NETBIRD_MANAGEMENT_URL/api/groups)
      GROUP_ID=$(echo "$GROUP_RESPONSE" | jq -r '.id')
  fi
  echo "$GROUP_ID"
}

create_setup_key_for_group() {
  GROUP_NAME="$1"

  GROUP_ID=$(group_create "$GROUP_NAME")
  RESPONSE_SETUP_KEY=$(curl -s -H "Authorization: Bearer $TOKEN" -d "{\"name\":\"$GROUP_NAME\",\"auto_groups\":[\"$GROUP_ID\"],\"type\":\"reusable\",\"expires_in\":31536000,\"usage_limit\":0}" $NETBIRD_MANAGEMENT_URL/api/setup-keys)
  NETBIRD_SETUP_KEY_TESTENV_GROUP=$(echo "$RESPONSE_SETUP_KEY" | jq -r '.key')
  echo "$NETBIRD_SETUP_KEY_TESTENV_GROUP"
}

policy_list() {
  POLICIES=$(curl -s -H "Authorization: Bearer $TOKEN" $NETBIRD_MANAGEMENT_URL/api/policies)
  echo "$POLICIES"
}

policy_list_names() {
  POLICIES=$(policy_list)
  echo "$POLICIES" | jq -r '.[].name'
}


policy_id_by_name() {
  POLICY_NAME="$1"

  POLICIES=$(policy_list)
  POLICY_ID=$(echo "$POLICIES" | jq -sr ".[] | map(select(.name==\"${POLICY_NAME}\")) | first | .id" )
  if [ "$POLICY_ID" == "null" ]; then
    echo ""
  else
    echo "$POLICY_ID"
  fi
}

policy_disable_default_rule() {
  DEFAULT_POLICY_ID=$(policy_id_by_name "Default")
  GROUP_ID=$(group_id_get_by_name "All")

  if [ -n "$DEFAULT_POLICY_ID" ]; then
    curl -qs -XPUT -H "Authorization: Bearer $TOKEN" \
          -H 'Content-Type application/json' \
          $NETBIRD_MANAGEMENT_URL/api/policies/"$DEFAULT_POLICY_ID" \
          --output /dev/null \
          --data-binary @- << EOF
  {
    "name": "Default",
    "description": "This is a default rule that allows connections between all the resources",
    "enabled": false,
    "query": "",
    "rules": [
      {
        "name": "Default",
        "description": "This is a default rule that allows connections between all the resources",
        "enabled": false,
        "sources": [
          "$GROUP_ID"
        ],
        "destinations": [
          "$GROUP_ID"
        ],
        "bidirectional": true,
        "protocol": "all",
        "ports": [],
        "action": "accept"
      }
    ]
  }
EOF
  fi
}

policy_create_rule() {
  POLICY_NAME="$1"
  GROUP_NAME="$2"

  POLICY_ID=$(policy_id_by_name "$POLICY_NAME")
  GROUP_ID=$(group_id_get_by_name "$GROUP_NAME")
  echo Found policy with POLICY_ID="$POLICY_ID"
  echo Found group with GROUP_ID="$GROUP_ID"

  if [ -z "$POLICY_ID" ]; then
    echo "Creating policy $POLICY_NAME"
    curl -s -XPOST -H "Authorization: Bearer $TOKEN" \
          -H 'Content-Type application/json' \
          $NETBIRD_MANAGEMENT_URL/api/policies \
          --output /dev/null \
          --data-binary @- << EOF
          {
            "name": "$POLICY_NAME",
            "description": "",
            "enabled": true,
            "rules": [
              {
                "name": "$POLICY_NAME",
                "description": "",
                "enabled": true,
                "sources": [
                  "$GROUP_ID"
                ],
                "destinations": [
                  "$GROUP_ID"
                ],
                "bidirectional": true,
                "protocol": "all",
                "action": "accept"
              }
            ]
          }
EOF
  else
    echo "Policy $POLICY_NAME already exists"
  fi

}

