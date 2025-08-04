#! /usr/bin/env bash

set -e

HOST=${HOST:-http://localhost:8180}
REALM=${REALM:-members}
ADMIN_REALM=${ADMIN_REALM:-master}
ADMIN_CLIENT_ID=${ADMIN_CLIENT_ID:-admin-cli}
ADMIN_USER=${ADMIN_USER:-admin}
ADMIN_PASSWORD=${ADMIN_PASSWORD:-admin}

if [ "$#" -lt 2 ] || [ "$#" -gt 5 ]; then
    echo "Usage: $0 <username> <password> [email] [firstName lastName]"
    exit 1
fi

NEW_USERNAME=$1
NEW_PASSWORD=$2

RANDOM_STRING=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 10 | head -n 1)
NEW_EMAIL="$RANDOM_STRING@test.cz"
FIRST_NAME="Test"
LAST_NAME="Test"

[ "$#" -ge 3 ] && NEW_EMAIL=$3
[ "$#" -ge 4 ] && FIRST_NAME=$4
[ "$#" -ge 5 ] && LAST_NAME=$5

echo "Attempting to get admin token"
TOKEN=$(curl -s -X POST \
  "$HOST/realms/$ADMIN_REALM/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$ADMIN_USER" \
  -d "password=$ADMIN_PASSWORD" \
  -d "grant_type=password" \
  -d "client_id=$ADMIN_CLIENT_ID" | jq -r '.access_token')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    echo "Failed to get admin token. Check your admin credentials and that Keycloak is running."
    exit 1
fi

echo "Admin token obtained successfully."

echo "Creating user '$NEW_USERNAME' in realm '$REALM'"

USER_JSON=$(cat <<EOF
{
  "username": "$NEW_USERNAME",
  "enabled": true,
  "email": "$NEW_EMAIL",
  "emailVerified": true,
  "firstName": "$FIRST_NAME",
  "lastName": "$LAST_NAME",
  "credentials": [{
    "type": "password",
    "value": "$NEW_PASSWORD",
    "temporary": false
  }]
}
EOF
)

CREATE_USER_RESPONSE=$(curl -s -X POST \
  "$HOST/admin/realms/$REALM/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer $TOKEN" \
  -d "$USER_JSON" --write-out "%{http_code}" --output /dev/null)

if [ "$CREATE_USER_RESPONSE" -eq 201 ]; then
  echo "User '$NEW_USERNAME' created successfully."
else
  echo "Failed to create user. Response code: $CREATE_USER_RESPONSE"
  curl -s -X POST \
    "$HOST/admin/realms/$REALM/users" \
    -H "Content-Type: application/json" \
    -H "Authorization: bearer $TOKEN" \
    -d "$USER_JSON"
  exit 1
fi

USER_ID=$(curl -s -X GET \
  "$HOST/admin/realms/$REALM/users?exact=true&username=$NEW_USERNAME" \
  -H "Authorization: bearer $TOKEN" \
  -H "Content-Type: application/json" | jq -r '.[0].id')

if [ -z "$USER_ID" ] || [ "$USER_ID" == "null" ]; then
    echo "Failed to get user ID for '$NEW_USERNAME'."
    exit 1
fi

echo "User ID for '$NEW_USERNAME' is '$USER_ID'."

GROUP_NAME="orca-admin"
GROUP_ID=$(curl -s -X GET \
  "$HOST/admin/realms/$REALM/groups" \
  -H "Authorization: bearer $TOKEN" \
  -H "Content-Type: application/json" | jq -r --arg GNAME "$GROUP_NAME" '.[] | select(.name==$GNAME) | .id')

if [ -z "$GROUP_ID" ] || [ "$GROUP_ID" == "null" ]; then
    echo "Failed to get group ID for '$GROUP_NAME'."
    echo "Please ensure group '$GROUP_NAME' exists in realm '$REALM'."
    exit 1
fi

echo "Group ID for '$GROUP_NAME' is '$GROUP_ID'."

ADD_TO_GROUP_RESPONSE=$(curl -s -X PUT \
  "$HOST/admin/realms/$REALM/users/$USER_ID/groups/$GROUP_ID" \
  -H "Authorization: bearer $TOKEN" \
  --write-out "%{http_code}" --output /dev/null)

if [ "$ADD_TO_GROUP_RESPONSE" -ge 200 ] && [ "$ADD_TO_GROUP_RESPONSE" -lt 300 ]; then
  echo "User '$NEW_USERNAME' added to group '$GROUP_NAME' successfully."
else
  echo "Failed to add user to group. Response code: $ADD_TO_GROUP_RESPONSE"
  exit 1
fi
