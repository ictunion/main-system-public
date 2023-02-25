# Orca

__OR__ganization __C__ontact __A__cquirement

> The orca or killer whale (Orcinus orca) is a [toothed whale](https://en.wikipedia.org/wiki/Toothed_whale)
> belonging to the [oceanic dolphin](https://en.wikipedia.org/wiki/Oceanic_dolphin) family, of which it is the largest member.
> It is the only [extant](https://en.wikipedia.org/wiki/Neontology#Extant_taxa_versus_extinct_taxa)
> species in the genus [Orcinus](https://en.wikipedia.org/wiki/Orcinus)
> and is recognizable by its black-and-white patterned body.

Orca is registration processor and members management system with [Web API](https://en.wikipedia.org/wiki/Web_API).
Orca web server based service build in [Rust](rust-lang.org) programming language.

## Setup

Orca connects to database maintained by [Gray Whale](../gray-whale).
Make sure you have configured gray whale before starting to work with Orca.

### Dependecies

Orca uses [Cargo](https://doc.rust-lang.org/cargo/) as a build tool.
Main dependecies are:

- [Rocket](https://rocket.rs/) web server framework
- [sqlx](https://crates.io/crates/sqlx) sql driver

__Toolchain:__

We use stable toolchain for rust. If you're [rustup](https://rustup.rs/) user make sure to update the stable toolchain

```
rustup default stable
```

or that your toolchain is up to date:

```
rustup update
```

__Run project:__

```
cargo run
```
