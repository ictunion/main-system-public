import getpass
import urllib.request
import urllib.parse
import urllib.error
import json
import sys

HOST = "http://localhost:8180"
REALM = "members"
CLIENT_ID = "orca"

username = input("Enter username: ")
password = getpass.getpass("Enter password: ")
print("")

data = urllib.parse.urlencode({
    "grant_type": "password",
    "client_id": CLIENT_ID,
    "username": username,
    "password": password,
}).encode()
url = f"{HOST}/realms/{REALM}/protocol/openid-connect/token"
with urllib.request.urlopen(urllib.request.Request(url, data=data)) as resp:
    response = json.loads(resp.read().decode())

jwt = response.get("access_token")
if not jwt:
    print("Failed to get JWT token")
    sys.exit(1)

print("JWT token obtained")

SIGNATURE = "data:image/png;base64,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"
LOCALE = "cs"

emails = ["jan.novak@example.cz", "marie.svobodova@example.cz", "petr.dvorak@example.cz", "lucie.horakova@example.cz", "tomas.kral@example.cz", "eva.prochazkova@example.cz", "martin.cerny@example.cz", "jana.blahova@example.cz", "ondrej.vesely@example.cz", "katerina.mala@example.cz"]
first_names = ["Jan", "Petr", "Marie", "Lucie", "Martin", "Jana", "Tomáš", "Eva", "Jiří", "Anna"]
last_names = ["Novák", "Svobodová", "Dvořák", "Horáková", "Král", "Procházková", "Černý", "Bláhová", "Veselý", "Malá"]
dates_of_births = ["1985-03-12", "1990-07-22", "1978-11-05", "1982-01-30", "1995-09-14", "2000-05-02", "1988-12-25", "1973-06-18", "1992-04-10", "1980-08-08"]
phone_numbers = ["+420 601 111 001", "+420 602 222 002", "+420 603 333 003", "+420 604 444 004", "+420 605 555 005", "+420 606 666 006", "+420 607 777 007", "+420 608 888 008", "+420 609 999 009", "+420 610 000 010"]
addresses = ["Náměstí Míru 1", "Dlouhá 42", "Husova 7", "Komenského 15", "Masarykova 3", "Nádražní 8", "Palackého 22", "Česká 5", "Štefánikova 19", "Husitská 33"]
cities = ["Praha", "Brno", "Ostrava", "Plzeň", "Olomouc", "Liberec", "Hradec Králové", "České Budějovice", "Zlín", "Pardubice"]
postal_codes = ["12000", "60200", "70200", "30100", "77900", "46001", "50002", "37001", "76001", "53002"]
occupations = ["Softwarový inženýr", "Učitelka", "Projektový manažer", "Grafická designérka", "Konzultant", "Obchodní ředitelka", "Lékař", "Fotografka", "DevOps inženýr", "Účetní"]
companies = ["Novák s.r.o.", "Microsoft", "Dvořák Tech a.s.", "Expedia", "Král Consulting", "EcoStore s.r.o.", "RWS", "Bláhová Design", "Veselý IT", "Infosys"]

limit = 10
print(f"Calling join API {limit} times...")

for email, first_name, last_name, date_of_birth, phone_number, address, city, postal_code, occupation, company in zip(
    emails, first_names, last_names, dates_of_births, phone_numbers,
    addresses, cities, postal_codes, occupations, companies
):
    payload = json.dumps({
        "email": email,
        "first_name": first_name,
        "last_name": last_name,
        "date_of_birth": date_of_birth,
        "address": address,
        "city": city,
        "postal_code": postal_code,
        "phone_number": phone_number,
        "company_name": company,
        "occupation": occupation,
        "signature": SIGNATURE,
        "local": LOCALE,
    }, ensure_ascii=False).encode()

    req = urllib.request.Request(
        "http://localhost:8000/registration/join",
        data=payload,
        headers={"Content-Type": "application/json", "x-real-ip": "127.0.0.1"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req) as resp:
            print(resp.status)
    except urllib.error.HTTPError as e:
        print(f"{e.code}: {e.read().decode()}")

workplace_names = ["First Testing Workplace", "Second Testing Workplace", "Third Testing Workplace"]
workplace_emails = ["first-testing-workplace@example.cz", "second-testing-workplace@example.cz", "third-testing-workplace@example.cz"]
workplace_group_ids = ["2365815d-554b-4431-9fd7-65fa1485064e", "5ac6ada1-ce8d-4bbd-a66b-40780735765b", "9a83d2b8-c079-44c2-8724-da67282dc3af"]

print(f"Calling create workplace API {len(workplace_names)} times...")

for name, email, group_id in zip(workplace_names, workplace_emails, workplace_group_ids):
    payload = json.dumps({
        "name": name,
        "email": email,
        "keycloak_group_id": group_id,
    }, ensure_ascii=False)).encode()

    req = urllib.request.Request(
        "http://localhost:8000/workplaces",
        data=payload,
        headers={"Content-Type": "application/json", "Authorization": f"Bearer {jwt}"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req) as resp:
            print(resp.status)
    except urllib.error.HTTPError as e:
        print(f"{e.code}: {e.read().decode()}")

