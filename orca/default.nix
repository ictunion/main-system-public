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
# TODO: add wrapProgram to configure path to tex
rustPlatform.buildRustPackage {
  inherit buildFeatures;

  pname = "ict-union-orca";
  version = "0.1.0";
  src = nix-gitignore.gitignoreSource [] ./.;
  cargoSha256 = "sha256-YZUV3la7cpk2eW/yWJueEaM721gdTALQ6OJ421tPBpw=";

  nativeBuildInputs = [ pkg-config makeWrapper ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
  postInstall = ''
    wrapProgram "$out/bin/orca" \\
      --suffix PATH : "${tex}/bin" \\
      --suffix OSFONTDIR : "${ibm-plex}/share/fonts/opentype"
  '';
}
