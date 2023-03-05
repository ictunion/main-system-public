# Main System

This is what we use to run ICT Union.

Overall system is a collection of sub systems used to manage
and run the system of the union organization.
We're following a naming scheme where every sub system of
overall system is named after species of [whales](https://en.wikipedia.org/wiki/Whale).

This helps us to have much finer grain control over permissions.
For instance [Gray Whale](gray-whale) -- out database migration manager --
expects super user level permissions to db
while [Orca](orca) -- our registration and members management service --
requires just permissions for reading and writing the data (no alterations of schema of db).

## Whales (sub-systems)

Every sub-system has its own documentation within README.md file!

| name                     | status                                                                                                      | role                                | description                                              |
|--------------------------|-------------------------------------------------------------------------------------------------------------|-------------------------------------|----------------------------------------------------------|
| [Gray Whale](gray-whale) | ![status](https://github.com/ictunion/main-systemn/actions/workflows/gray-whale.yaml/badge.svg?branch=main) | Database migrations and management  | Gray Whales have longest migrations of any marine mammal |
| [Orca](orca)             | ![status](https://github.com/ictunion/main-systemn/actions/workflows/orca.yaml/badge.svg?branch=main)       | Registration and members management | ORganization Contact Acquirement                         |

## Running

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
