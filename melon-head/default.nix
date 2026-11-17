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
  npmDepsHash = "sha256-D4WT9dEg+1RSA9GiInoi9HW5OpF01eglaBDCpwXwMAc=";

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
