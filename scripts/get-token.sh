#! /usr/bin/env nix-shell
#! nix-shell -i bash -p curl jq

HOST=https://keycloak.ictunion.cz
REALM=testing-members

>&2 echo " This script will return acess token to testing-members instance"
read -p "Enter client-id of your app: " CLIENTID
read -p "Enter user name: " USERNAME
read -s -p "Enter password: " PWD
>&2 echo ""

RESPONSE=$(curl --silent -d "grant_type=password&client_id=$CLIENTID&username=$USERNAME&password=$PWD" "$HOST/realms/$REALM/protocol/openid-connect/token")

>&2 echo "API Response is"
>&2 echo $RESPONSE
>&2 echo ""

ACCESS_TOKEN=$(echo $RESPONSE | jq -r '.access_token')
>&2 echo "Access token is"
>&2 echo $ACCESS_TOKEN
>&2 echo ""

>&2 echo "Attempting to copy token to clipboard"
echo $ACCESS_TOKEN | xclip -sel clip 2> /dev/null # Linux X11
echo $ACCESS_TOKEN | clip.exe 2> /dev/null # Windows
echo $ACCESS_TOKEN | pbcopy 2> /dev/null # Macos
