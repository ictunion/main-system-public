{ rustPlatform
, pkg-config
, openssl
, stdenv
, darwin
, lib
, texlive
, makeWrapper
, nix-gitignore
, buildFeatures ? []
}:
let
  tex = import ./tex.nix { inherit texlive; };
in
# TODO: add wrapProgram to configure path to tex
rustPlatform.buildRustPackage {
  inherit buildFeatures;

  pname = "ict-union-orca";
  version = "0.1.0";
  src = nix-gitignore.gitignoreSource [] ./.;
  cargoSha256 = "sha256-+r8L9zZ66GHn7KVSyLfcbmCsatZVkEKFRDoRMDRPaH4=";

  nativeBuildInputs = [ pkg-config makeWrapper ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
  postInstall = ''
    wrapProgram "$out/bin/orca" --suffix PATH : "${tex}/bin"
  '';
}
