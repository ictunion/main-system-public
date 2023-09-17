# Melon Head

Web UI client for [Orca](../orca).

**Work in Progress**

## Developing

First make sure configuration file is present in the project.
You can for instance symlink example config or copy it if you need to make some changes
to make it work in your setup:

```
$ ln -s config.example.json config.json
```

We're using npm and npm scripts while working on project.

| command            | function                     |
| ------------------ | ---------------------------- |
| `npm install`      | Install dependencies         |
| `npm run clean`    | Clean parcel cache           |
| `npm run build`    | Build production assets      |
| `npm run format`   | Autoformat source code       |
| `npm start`        | Run development server       |
| `npm run watch`    | Run compiler in watch mode   |


## Nix Build

`$ nix-build test.nix`
