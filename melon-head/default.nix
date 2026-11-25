{ buildNpmPackage
, nodejs
, nix-gitignore
, python312
, config
}:
let
  config-json = builtins.toJSON config;
in
buildNpmPackage {
  name = "ict-union-melon-head";
  nativeBuildInputs = [
    nodejs
    python312
  ];
  src = nix-gitignore.gitignoreSource [] ./.;
  npmDepsHash = "sha256-pSOD8d0Nk8flCkqdwUqynidVJx3pOn4vu6DBo1tLN/Q=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  buildPhase = ''
    npm run build
  '';

  checkPhase = ''
    npx rescript format --check
  '';

  installPhase = ''
    mkdir -p $out/var/www
    cp -r dist/* $out/var/www
  '';
}
