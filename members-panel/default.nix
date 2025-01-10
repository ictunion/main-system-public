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
  npmDepsHash = "sha256-rfQ27j09/u3QsJW0BlpeQJrkUtwP1lKZtJ67/jsC1SM=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  installPhase = ''
    mkdir -p $out/var/www
    npm run build
    cp dist/* $out/var/www
  '';
}
