import sys
import getpass
import urllib.request
import urllib.parse
import json
import subprocess

HOST = "http://localhost:8180"
REALM = "members"

print("This script will return access token to testing-members instance", file=sys.stderr)
client_id = input("Enter client-id of your app: ")
username = input("Enter user name: ")
password = getpass.getpass("Enter password: ")
print("", file=sys.stderr)

data = urllib.parse.urlencode({
    "grant_type": "password",
    "client_id": client_id,
    "username": username,
    "password": password,
}).encode()
url = f"{HOST}/realms/{REALM}/protocol/openid-connect/token"

with urllib.request.urlopen(urllib.request.Request(url, data=data)) as resp:
    response = resp.read().decode()

print("API Response is", file=sys.stderr)
print(response, file=sys.stderr)
print("", file=sys.stderr)

access_token = json.loads(response)["access_token"]
print("Access token is", file=sys.stderr)
print(access_token, file=sys.stderr)
print("", file=sys.stderr)

print("Attempting to copy token to clipboard", file=sys.stderr)
for cmd in [["wl-copy"], ["xclip", "-sel", "clip"], ["clip.exe"], ["pbcopy"]]:
    try:
        subprocess.run(cmd, input=access_token.encode(), stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        pass
