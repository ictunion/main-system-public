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
  cargoSha256 = "sha256-TcQ26NRmd7aCok2WvRWZBmLGNI2bmQZEqxwiNT1ys9o=";

  nativeBuildInputs = [ pkg-config makeWrapper ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
  postInstall = ''
    wrapProgram "$out/bin/orca" --suffix PATH : "${tex}/bin"
  '';
}
