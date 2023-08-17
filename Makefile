DB_NAME:=ictunion
DB_SUPERUSER:=postgres
DB_PORT:=5432
DB_PASSWORD:=superuser
DB_HOST:=localhost
KEYCLOAK_URL:="https://keycloak.ictunion.cz"
KEYCLOAK_REALM:="testing-members"
JWT_SECRET=`cat keycloak-certs`
PGRST_JWT_ROLE_CLAIM_KEY:=".resource_access.postgrest.roles[0]"

keycloak-certs:
	curl $(KEYCLOAK_URL)/realms/$(KEYCLOAK_REALM)/protocol/openid-connect/certs > keycloak-certs

.PHONY: postgres
postgres:
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) \
		docker compose up postgres

.PHONY: postgrest
postgrest: keycloak-certs
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) JWT_SECRET=$(JWT_SECRET) PGRST_JWT_ROLE_CLAIM_KEY=$(PGRST_JWT_ROLE_CLAIM_KEY) \
		docker compose up postgres postgrest

.PHONY: up
up: keycloak-certs
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) JWT_SECRET=$(JWT_SECRET) PGRST_JWT_ROLE_CLAIM_KEY=$(PGRST_JWT_ROLE_CLAIM_KEY) \
		docker compose up

.PHONY: migrate
migrate:
	pushd gray-whale; refinery migrate -p ./migrations; popd;

.PHONY: psql
psql:
	PGPASSWORD=$(DB_PASSWORD) psql -h $(DB_HOST) -p $(DB_PORT) -d $(DB_NAME) -U $(DB_SUPERUSER)

.PHONY: clean
clean:
	$(RM) keycloak-certs
