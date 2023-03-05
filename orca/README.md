# Orca

**OR**ganization **C**ontact **A**cquirement

> The orca or killer whale (Orcinus orca) is a [toothed whale](https://en.wikipedia.org/wiki/Toothed_whale)
> belonging to the [oceanic dolphin](https://en.wikipedia.org/wiki/Oceanic_dolphin) family, of which it is the largest member.
> It is the only [extant](https://en.wikipedia.org/wiki/Neontology#Extant_taxa_versus_extinct_taxa)
> species in the genus [Orcinus](https://en.wikipedia.org/wiki/Orcinus)
> and is recognizable by its black-and-white patterned body.

Orca is registration processor and members management system with [Web API](https://en.wikipedia.org/wiki/Web_API).

Orca is web server based service build in [Rust](rust-lang.org) programming language.

## Setup

Orca connects to database maintained by [Gray Whale](../gray-whale).
Make sure you have configured gray whale before starting to work with Orca.

### Dependecies

Make sure you have these dependecies installed on your machine:

- [rustup](https://rustup.rs/) to manage rustc and cargo
- [open-ssl](https://www.openssl.org/) C library for TLS implementation
- [xelatex](https://xetex.sourceforge.net/) executable for printing PDF from TeX

### Configuration

Copy example configuration file

```
cp Rocket.example.toml Rocket.toml
```

## Developing

### Toolchain

We use stable toolchain for rust.
If you're using [rustup](https://rustup.rs/) you can ensure the right tooling using:

```
rustup default stable
```

or that your toolchain is up to date:

```
rustup update
```

### Libraries

Orca uses [Cargo](https://doc.rust-lang.org/cargo/) as a build tool.
Main dependecies are:

- [Rocket](https://rocket.rs/) web server framework
- [sqlx](https://crates.io/crates/sqlx) sql driver

__Generate Documentation__ for all dependecies and code:

```
cargo doc --open
```

### Running project

__Run server on your machine:__

```
cargo run
```

__Run tests:__

```
cargo test
```
