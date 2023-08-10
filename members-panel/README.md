# Members Panel

**Work In Progress!**

Access entry dashboard for organization members

## Setup

First define configuration starting from example file:

```
cp config.example.json example.json
```

Configuration is compile-time. It's used to produce static file where all the configuration options
are inlined. This means that configuration file is used to configure buid
prior to invoking compiler.

Start development server:

```
npm start
```

Compile production assets:

```
npm run build
```

## Nix

Trigger test nix build:

```
nix-build test.nix
```
