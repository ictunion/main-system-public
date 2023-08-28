{ buildNpmPackage
, nodejs
, nix-gitignore
, config
}:
let
  config-json = builtins.toJSON config;
in
buildNpmPackage {
  name = "administration-panel";
  buildInputs = [
    nodejs
  ];
  src = nix-gitignore.gitignoreSource [] ./.;
  npmDepsHash = "sha256-Kh+9ri5JnjO0fqZOZuQxA7gzHMYGHSrcKuKyvb+tjxA=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  installPhase = ''
    mkdir -p $out/var/www
    npm run build
    cp dist/* $out/var/www
  '';
}
