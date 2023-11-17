{ buildNpmPackage
, nodejs
, nix-gitignore
, python39
, config
}:
let
  config-json = builtins.toJSON config;
in
buildNpmPackage {
  name = "ict-union-melon-head";
  nativeBuildInputs = [
    nodejs
    python39
  ];
  src = nix-gitignore.gitignoreSource [] ./.;
  npmDepsHash = "sha256-PXvM1W2hOYzjRtccvNhPR1EqBkps+vsk5y19fXuS7sE=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  buildPhase = ''
    npm run build
  '';

  checkPhase = ''
    npx rescript format -all -check
  '';

  installPhase = ''
    mkdir -p $out/var/www
    cp -r dist/* $out/var/www
  '';
}
