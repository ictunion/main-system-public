{ buildNpmPackage
, nodejs
, nix-gitignore
, config
}:
let
  config-json = builtins.toJSON config;
in
buildNpmPackage {
  name = "members-panel";
  buildInputs = [
    nodejs
  ];
  src = nix-gitignore.gitignoreSource [] ./.;
  npmDepsHash = "sha256-R8cADTzF2o5YbtXISE5YYRj0tTwV7j/Z2+jq7JCPq+M=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  installPhase = ''
    mkdir -p $out/var/www
    npm run build
    cp dist/* $out/var/www
  '';
}
