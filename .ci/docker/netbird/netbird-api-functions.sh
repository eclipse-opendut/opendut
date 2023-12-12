#!/bin/bash

KEYCLOAK_URL=http://keycloak
NETBIRD_MANAGEMENT_URL=http://netbird-management

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

get_user_oauth_token() {
  # requires public client and client with password grant enabled, directAccessGrantsEnabled=true
  RESPONSE=$(curl -s -d "client_id=netbird-mgmt-cli" -d "username=netbird" -d "password=netbird" -d "grant_type=password" $KEYCLOAK_URL/realms/netbird/protocol/openid-connect/token)
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
    NETBIRD_USER_ID=$(echo "$USERS" | jq -r ".[] | select(.name==\"$USER_NAME\").id")
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
    echo "Waiting for $user_name to be available..."
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

