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
  npmDepsHash = "sha256-0oOi5EVRR6Fl9Zlp9XEkIeSP2P45JdBwm0V8RfWfspo=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  installPhase = ''
    mkdir -p $out/var/www
    npm run build
    cp dist/* $out/var/www
  '';
}
