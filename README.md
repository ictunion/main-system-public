# Main System

This is what we use to run ICT Union.

System is split into few components witch separated responsibilities.

This helps us to have much finer grain control over permissions.
For instance [Gray Whale](gray-whale) -- out database migration manager --
expects super user level permissions to db
while [Orca](orca) -- our registration and members management service --
requires just permissions necessary for its operation.

## Components

Every component has its own documentation within README.md file!

| name                                         | status                                                                                                               | role                                      | language   |
|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------|-------------------------------------------|------------|
| Acceptance Test                              | ![status](https://github.com/ictunion/main-system/actions/workflows/acceptance-test.yaml/badge.svg?branch=main)      | Integration test of whole system          | --         |
| [Gray Whale](gray-whale)                     | ![status](https://github.com/ictunion/main-system/actions/workflows/gray-whale.yaml/badge.svg?branch=main)           | Database migrations and management        | SQL        |
| [Orca](orca)                                 | ![status](https://github.com/ictunion/main-system/actions/workflows/orca.yaml/badge.svg?branch=main)                 | Registration and onboarding processing    | Rust       |
| [Melon Head](melon-head)                     | ![status](https://github.com/ictunion/main-system/actions/workflows/melon-head.yaml/badge.svg?branch=main)           | Web UI client for orca                    | Rescript   |
| [Member's Panel](members-panel)              | ![status](https://github.com/ictunion/main-system/actions/workflows/members-panel.yaml/badge.svg?branch=main)        | Entry dashboard for organization members  | TypeScript |
| [Administration Panel](administration-panel) | ![status](https://github.com/ictunion/main-system/actions/workflows/administration-panel.yaml/badge.svg?branch=main) | Administration tool for member management | TypeScript |

## Goals

The goal behind this project is to eventually aggregate all systems that help
with organization management starting with system for new member registrations.
We use service oriented architecture but with monolithic [Postgresql](https://www.postgresql.org/).
Isolation is of db layer is done purely by capabilities provided by the DB.
This is so that we could leverage unified data source for providing various functionality.
But it also means we will need to be super careful about coupling database schema into services.

Primary motivation for service oriented architecture is that this imposes little to no restriction
on what technical implementation of any individual part. That should hopefully in turn reduce
the barrier for entry as individual contributors have a free hand in choosing familiar stack for
implementation of individual services.

We choose to use Rust for the first service supporting the registrations since it is community driven,
focused on producing safe and fast goal and community governance is fairly compatible with union internal
guidelines.

Technical solution is also guided by principles of:

- Limiting amount of dependencies on corporate owned proprietary technology
- Focus on cheap running costs of the system
- Focus on sustainability in terms of maintenance cost, environmental impact etc.

## Working with the Project

We define [Makefile](Makefile) for convenient workflow around the project.

### Start Database

```
make postgres
```

### Migrate Database

__Make sure to configure [Gray Whale](gray-whale) according to its documentation first!__

```
make migrate
```

### Explore Database

```
make psql
```

### Start HTTP API Service Using Postgrest

```
make postgrest
```

#### Miscellaneous Make Target

Generate `keycloak-certs` file by fetching certs from keycloak instance:

```
make keycloak-certs
```

Clean all outputs from make

```
make clean
```

### Start all services

```
make up
```

## Nix

[flake.nix](https://nixos.wiki/wiki/Flakes) so don't forget to make sure that you have [support for flakes](https://nixos.wiki/wiki/Flakes) enabled in your config.

Flake exports dev various components as packages and their corresponding dev shells. See examples bellow.

```bash
# Run migrations using nix
$ nix run .#gray-whale -- migrate

# Build orca using nix
$ nix build .#orca

# jump into dev shell for orca
$ cd orca
$ nix develop .#orca
```

## License

All source code is releleased under [AGPLv3](LICENSE) license unless specifically state otherwise.

````
    Copyright (C) 2023 members of Odborová organizace pracujících v ICT

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
````
