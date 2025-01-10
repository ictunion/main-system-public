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
  npmDepsHash = "sha256-5btV7tCR9rt09bECCKoNs1b1K4h+opjLtZg1Rf7/Fp4=";

  configurePhase = ''
    echo '${config-json}' > config.json
  '';

  installPhase = ''
    mkdir -p $out/var/www
    npm run build
    cp dist/* $out/var/www
  '';
}
