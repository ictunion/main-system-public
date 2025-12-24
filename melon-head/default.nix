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
  npmDepsHash = "sha256-VuxoJd5xVti1qle0i26+fWvpA6WNu3ToSKGJfAedo8M=";

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
