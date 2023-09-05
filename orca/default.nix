{ rustPlatform
, pkg-config
, openssl
, stdenv
, darwin
, lib
, texlive
, makeWrapper
, nix-gitignore
, ibm-plex
, buildFeatures ? []
}:
let
  tex = import ./latex { inherit texlive; };
in
rustPlatform.buildRustPackage {
  inherit buildFeatures;

  pname = "ict-union-orca";
  version = "0.1.0";
  src = nix-gitignore.gitignoreSource [] ./.;
  cargoSha256 = "sha256-JGSLmdUXhhB0z8x2w1feKnHUGb7vWviJBP9dbq0E9aY=";

  nativeBuildInputs = [ pkg-config makeWrapper ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
  postInstall = ''
    wrapProgram "$out/bin/orca" --suffix PATH : "${tex}/bin"
  '';
}
