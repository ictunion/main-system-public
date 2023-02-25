DB_NAME:=ictunion
DB_SUPERUSER:=postgres
DB_PORT:=5432
DB_PASSWORD:=superuser
DB_HOST:=localhost

.PHONY: postgres
postgres:
	DB_USER=$(DB_SUPERUSER) DB_NAME=$(DB_NAME) DB_PASSWORD=$(DB_PASSWORD) \
		docker compose up postgres

.PHONY: migrate
migrate:
	pushd gray-whale; refinery migrate -p ./migrations; popd;

.PHONY: psql
psql:
	PGPASSWORD=$(DB_PASSWORD) psql -h $(DB_HOST) -p $(DB_PORT) -d $(DB_NAME) -U $(DB_SUPERUSER)
