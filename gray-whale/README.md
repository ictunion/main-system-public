# Gray Whale

> [Graw whales](https://en.wikipedia.org/wiki/Gray_whale) are thought to have the longest migrations of any marine mammal,
> traveling 10,000-12,000 miles round trip between their breeding grounds off Baja California
> to their feeding grounds in the Bering and Chukchi Seas off Alaska and Russia.
> [A gray whale reported in 2015](https://www.livescience.com/50487-western-gray-whale-migration.html)
> broke all marine mammal migration records
> - she traveled from Russia to Mexico and back again.
> this was a distance of 13,988 miles in 172 days.

Gray Whale is database migration system used to manage schema add access of our [PostgresSQL](https://www.postgresql.org/) database.

## Local Database

You'll need to run PostgreSQL database server on your system. This can be achieved in one of many ways
from running PostgreSQL as a systemd service on Linux system, launchd service on MacOS or using Windows installers.

[Makefile](../Makefile) in the root of the project provides convenient multiplatform way of running postgres
inside a [Docker](https://www.docker.com/) container.

## Using Refinery Cli

Use [Refinery Cli](https://github.com/rust-db/refinery) tool to manage database schema.
See [official documentation](https://crates.io/crates/refinery_cli) for installation instructions.

## Configure

Copy example configuration:

```
cp refinery.example.toml refinery.toml
```

If you're using provided docker based solution for starting Postgres (make postgres)
you're all set. If you use other way to run postgres __you will most likely need to edit the refinery.toml__
to much your configuration. Alternatively you can also avoid starting from example file and
simply use `refinery configure` to produce config for your setup.

## Create a database

using psql create database called `ictunion`.

> Using makefile from root of the repository psql could be started using make:
>
> ```
> make psql DB_NAME=postgres
> ````


```
CREATE DATABASE itctunion;
```

## Migrate Database

**migrate database to new schema:**

```bash
refinery migrate
```
