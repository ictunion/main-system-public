DB_NAME:=ictunion
DB_SUPERUSER:=postgres
DB_PORT:=5432
DB_PASSWORD:=superuser
DB_HOST:=localhost
KEYACLOAK_USER:=admin
KEYCLOAK_PASSWORD:=admin

orca/result:
	pushd orca && nix build

.PHONY: postgres
postgres:
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) \
		docker compose up postgres

.PHONY: up
up:
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) JWT_SECRET=$(JWT_SECRET) PGRST_JWT_ROLE_CLAIM_KEY=$(PGRST_JWT_ROLE_CLAIM_KEY) \
    KEYCLOAK_USER=$(KEYCLOAK_USER) KEYCLOAK_PASSWORD=$(KEYCLOAK_PASSWORD) \
		docker compose up

.PHONY: migrate
migrate:
	pushd gray-whale; refinery migrate -p ./migrations; popd;

.PHONY: psql
psql:
	PGPASSWORD=$(DB_PASSWORD) psql -h $(DB_HOST) -p $(DB_PORT) -d $(DB_NAME) -U $(DB_SUPERUSER)

.PHONY: orca
orca: orca/result
	pushd orca && ./result/bin/orca

.PHONY: clean
clean:
	$(RM) keycloak-certs
	$(RM) orca/result
