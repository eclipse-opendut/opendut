#!/bin/bash

wait_for_keycloak() {
  # wait until keycloak is ready and returns a status code < 400
  while ! curl --silent --fail "$KEYCLOAK_URL" --output /dev/null; do
    echo "Waiting for Keycloak to start up..."
    sleep 5
  done
  echo "Keycloak ready"
}
kcadm() { local cmd="$1" ; shift ; "$KCADM_PATH" "$cmd" --config /tmp/kcadm.config "$@" ; }
kcauth() { kcadm config credentials config --server "$KEYCLOAK_URL" --realm master --user "$KEYCLOAK_ADMIN" --password "$KEYCLOAK_ADMIN_PASSWORD" ; }

list_realms() {
  kcadm get realms 2>/dev/null | jq -r ".[].realm"
}

create_realm() {
  realm_name="$1"
  EXISTING_REALMS=$(list_realms)
  if [[ "$EXISTING_REALMS" == *"${realm_name}"* ]]; then
    echo "Realm ${realm_name} already exists"
  else
    kcadm create realms -s realm="${realm_name}" -s enabled=true
  fi
}

create_user() {
  USER_NAME="$1"
  USER_PASSWORD="$2"
  USER_GROUP="$3"
  USER_ROLE="$4"
  USER_REALM="${5:-$REALM}"

  echo "create user ${USER_NAME}"
  kcadm create users -r "${USER_REALM}" -f - << EOF
  {
    "username": "${USER_NAME}",
    "email": "${USER_NAME}@example.com",
    "firstName": "Firstname",
    "lastName": "Lastname",
    "requiredActions": [],
    "emailVerified": false,
    "groups": [],
    "attributes": {},
    "enabled": true
  }
EOF
  USER_ID=$(kcadm get users -r "${USER_REALM}" | jq -r ".[] | select(.username==\"${USER_NAME}\").id")

  echo "update user password for user \'${USER_NAME}\'"
  kcadm update users/"$USER_ID"/reset-password -r "${USER_REALM}" -f - << EOF
  {
    "temporary": false,
    "type": "password",
    "value": "$USER_PASSWORD"
  }
EOF

  if [ -n "$USER_GROUP" ]; then
    echo "add user to group"
    USER_GROUP_ID=$(kcadm get groups | jq -r ".[] | select(.name==\"${USER_GROUP}\").id")
    kcadm update users/"$USER_ID"/groups/"$USER_GROUP_ID" -r "${USER_REALM}"
  fi

  if [ -n "$USER_ROLE" ]; then
    echo "add user to role"
    USER_ROLE_ID=$(kcadm get roles -r "${USER_REALM}" | jq -r ".[] | select(.name==\"${USER_ROLE}\").id")
    USER_ROLE_CONTAINER_ID=$(kcadm get users/"$USER_ID"/role-mappings -r "${USER_REALM}" | jq -r '.realmMappings[] | select(.name=="default-roles-master").containerId')
    kcadm create users/"$USER_ID"/role-mappings/realm -r "${USER_REALM}" -f - << EOF
    [{
      "id": "$USER_ROLE_ID",
      "name": "$USER_ROLE",
      "description": "",
      "composite": false,
      "clientRole": false,
      "containerId": "$USER_ROLE_CONTAINER_ID"
    }]
EOF
  fi
}

list_realm_roles() {
  REALM=${1:-$REALM}
  kcadm get roles -r "${REALM}" 2>/dev/null | jq -r ".[].name"
}

create_realm_role() {
  ROLE_NAME="$1"
  ROLE_REALM="${2:-$REALM}"

  EXISTING_REALM_ROLES=$(list_realm_roles "${ROLE_REALM}")
  if [[ "$EXISTING_REALM_ROLES" == *"${ROLE_NAME}"* ]]; then
    echo "Realm ${ROLE_NAME} already exists"
  else
      echo "Create realm role ${ROLE_NAME}"
      kcadm create roles -r "${ROLE_REALM}" -f - << EOF
      {
        "name": "${ROLE_NAME}",
        "description": "${ROLE_NAME} role",
        "attributes": {}
      }
EOF
  fi
}

get_client_id() {
  CLIENT_NAME="$1"
  CLIENT_REALM="${2:-$REALM}"

  kcadm get clients -r "${CLIENT_REALM}" | jq -r ".[] | select(.clientId==\"${CLIENT_NAME}\").id"
}

create_public_client() {
  CLIENT_NAME="$1"
  CLIENT_REDIRECT_URI="$2"
  CLIENT_REALM="${3:-$REALM}"

  CLIENT_EXISTS=$(get_client_id "${CLIENT_NAME}" "${CLIENT_REALM}")
  if [ -z "$CLIENT_EXISTS" ]; then
    echo "Create client ${CLIENT_NAME}"
    kcadm create clients -r "${CLIENT_REALM}" -f - << EOF
      {
        "enabled": true,
        "clientId": "$CLIENT_NAME",
        "publicClient": true,
        "standardFlowEnabled": true,
        "fullScopeAllowed": true,
        "webOrigins": ["*"],
        "redirectUris": [$CLIENT_REDIRECT_URI],
        "attributes": {
          "access.token.lifespan": "300",
          "post.logout.redirect.uris": "+"
        }
      }
EOF
    echo "Client create result: $?"

  else
    echo "Client ${CLIENT_NAME} already exists in realm ${CLIENT_REALM}."
    return
  fi

}

create_public_client_with_direct_access() {
  CLIENT_NAME="$1"
  CLIENT_REDIRECT_URI="$2"
  CLIENT_REALM="${3:-$REALM}"

  CLIENT_EXISTS=$(get_client_id "${CLIENT_NAME}" "${CLIENT_REALM}")
  if [ -z "$CLIENT_EXISTS" ]; then
    echo "Create client ${CLIENT_NAME}"
    kcadm create clients -r "${CLIENT_REALM}" -f - << EOF
      {
        "protocol": "openid-connect",
        "clientId": "$CLIENT_NAME",
        "name": "",
        "description": "",
        "publicClient": true,
        "authorizationServicesEnabled": false,
        "serviceAccountsEnabled": false,
        "implicitFlowEnabled": false,
        "directAccessGrantsEnabled": true,
        "standardFlowEnabled": true,
        "fullScopeAllowed": true,
        "frontchannelLogout": true,
        "attributes": {
          "saml_idp_initiated_sso_url_name": "",
          "oauth2.device.authorization.grant.enabled": false,
          "oidc.ciba.grant.enabled": false
        },
        "alwaysDisplayInConsole": false,
        "rootUrl": "http://netbird-ui",
        "baseUrl": "http://netbird-ui",
        "redirectUris": [$CLIENT_REDIRECT_URI],
        "webOrigins": [
          "*"
        ]
      }
EOF
    echo "Public client created. Result: $?"

  else
    echo "Client ${CLIENT_NAME} already exists in realm ${CLIENT_REALM}."
    return
  fi

}

client_delete() {
  CLIENT_NAME="$1"
  CLIENT_REALM="${2:-$REALM}"

  CLIENT_ID=$(get_client_id "${CLIENT_NAME}" "${CLIENT_REALM}")
  kcadm delete clients/"$CLIENT_ID" -r "${CLIENT_REALM}"
}

create_secret_client() {
  CLIENT_NAME="$1"
  CLIENT_SECRET="$2"
  CLIENT_REALM="${3:-$REALM}"

  # see https://stackoverflow.com/questions/66374537/keycloak-set-static-client-secret

  CLIENT_EXISTS=$(get_client_id "${CLIENT_NAME}" "${CLIENT_REALM}")
  if [ -z "$CLIENT_EXISTS" ]; then
    echo "Create client ${CLIENT_NAME}"
    kcadm create clients -r "${CLIENT_REALM}" -f - << EOF
      {
        "protocol": "openid-connect",
        "clientId": "$CLIENT_NAME",
        "name": "",
        "description": "",
        "publicClient": false,
        "authorizationServicesEnabled": false,
        "serviceAccountsEnabled": true,
        "implicitFlowEnabled": false,
        "secret": "$CLIENT_SECRET",
        "directAccessGrantsEnabled": true,
        "standardFlowEnabled": true,
        "frontchannelLogout": true,
        "attributes": {
          "saml_idp_initiated_sso_url_name": "",
          "oauth2.device.authorization.grant.enabled": true,
          "oidc.ciba.grant.enabled": false
        },
        "alwaysDisplayInConsole": false,
        "rootUrl": "",
        "baseUrl": ""
      }
EOF
    echo "Secret client created. Result: $?"

    else
      echo "Client ${CLIENT_NAME} already exists in realm ${CLIENT_REALM}."
      return
    fi

}

get_client_scope_id() {
    CLIENT_SCOPE_NAME="$1"
    CLIENT_SCOPE_REALM="${2:-$REALM}"

  kcadm get client-scopes -r "${CLIENT_SCOPE_REALM}" | jq -r ".[] | select(.name==\"$CLIENT_SCOPE_NAME\").id"
}

create_client_scope() {
  CLIENT_SCOPE_NAME="$1"
  CLIENT_SCOPE_TYPE="${2:-default}"
  CLIENT_SCOPE_REALM="${3:-$REALM}"

  kcadm create client-scopes -r "${CLIENT_SCOPE_REALM}" -f - << EOF
    {
      "name": "$CLIENT_SCOPE_NAME",
      "description": "",
      "attributes": {
        "consent.screen.text": "",
        "display.on.consent.screen": "true",
        "include.in.token.scope": "true",
        "gui.order": ""
      },
      "type": "$CLIENT_SCOPE_TYPE",
      "protocol": "openid-connect"
    }
EOF
}

client_scope_add_audience() {
  CLIENT_SCOPE_NAME="$1"
  CLIENT_SCOPE_AUDIENCE_NAME="$2"
  CLIENT_SCOPE_AUDIENCE_CLIENT_NAME="$3"
  CLIENT_SCOPE_REALM="${4:-$REALM}"

  CLIENT_SCOPE_ID=$(get_client_scope_id "$CLIENT_SCOPE_NAME" "$CLIENT_SCOPE_REALM")

  kcadm create client-scopes/"$CLIENT_SCOPE_ID"/protocol-mappers/models -r "${CLIENT_SCOPE_REALM}" -f - << EOF
   {
     "protocol": "openid-connect",
     "protocolMapper": "oidc-audience-mapper",
     "name": "$CLIENT_SCOPE_AUDIENCE_NAME",
     "config": {
       "included.client.audience": "$CLIENT_SCOPE_AUDIENCE_CLIENT_NAME",
       "included.custom.audience": "",
       "id.token.claim": "false",
       "access.token.claim": "true"
     }
  }
EOF

}

add_client_scope_to_client() {
  CLIENT_NAME="$1"
  CLIENT_SCOPE_NAME="$2"
  CLIENT_REALM="${3:-$REALM}"

  CLIENT_ID=$(get_client_id "$CLIENT_NAME" "$CLIENT_REALM")
  CLIENT_SCOPE_ID=$(get_client_scope_id "$CLIENT_SCOPE_NAME" "$CLIENT_REALM")
  kcadm update clients/"$CLIENT_ID"/default-client-scopes/"$CLIENT_SCOPE_ID" -r "${CLIENT_REALM}"
  # shellcheck disable=SC2181
  if [ $? -eq 0 ]; then
    echo "Added client scope $CLIENT_SCOPE_NAME to client $CLIENT_NAME"
  else
    echo "Failed to add client scope $CLIENT_SCOPE_NAME to client $CLIENT_NAME"
    return 1
  fi
}

client__assign_service_account_role() {
  CLIENT_NAME="$1"
  CLIENT_ROLE_CLIENT_ID="$2"
  CLIENT_ROLE_NAME="$3"

  CLIENT_REALM="${4:-$REALM}"
  kcadm add-roles -r "$CLIENT_REALM" --uusername service-account-"$CLIENT_NAME" --cclientid "$CLIENT_ROLE_CLIENT_ID" --rolename "$CLIENT_ROLE_NAME"
}

function add_github_enterprise_idp {
  GH_IDP_CLIENT_ID="$1"
  GH_IDP_CLIENT_SECRET="$2"
  GH_IDP_BASE_URL="$3"
  GH_IDP_API_URL="$4"

  echo "add github enterprise idp"
  kcadm create identity-provider/instances -r ${REALM} -f - << EOF
  {
    "config": {
      "clientId": "$GH_IDP_CLIENT_ID",
      "clientSecret": "$GH_IDP_CLIENT_SECRET",
      "guiOrder": "",
      "baseUrl": "$GH_IDP_BASE_URL",
      "apiUrl": "$GH_IDP_API_URL"
    },
    "providerId": "github-enterprise",
    "alias": "github-enterprise"
  }
EOF

}

function make_github_enterprise_idp_default {
    IDP_PROVIDER_ID=$(kcadm get authentication/flows/browser/executions | jq -r '.[] | select(.providerId=="identity-provider-redirector").id')
    echo "add github enterprise as default provider to browser flow"
    kcadm create authentication/executions/$IDP_PROVIDER_ID/config -r ${REALM} -f - << EOF
    {
      "alias": "github-enterprise",
      "config": {
        "defaultProvider": "github-enterprise"
      }
    }
EOF
}

