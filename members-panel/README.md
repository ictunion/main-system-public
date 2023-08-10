# Members Panel

**Work In Progress!**

Access entry dashboard for organization members.

## Setup

First define configuration starting from example file:

```
cp config.example.json example.json
```

Configuration happens in compile-time. All compilation options are inlined into artifacts during compilation.

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
