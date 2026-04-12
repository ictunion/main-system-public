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
  npmDepsHash = "sha256-nbt6I1DprqgqLDoMgMr3pDFv2gjb+4n8jUx9+4XV2ok=";

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
