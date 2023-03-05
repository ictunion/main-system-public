{ rustPlatform
, pkg-config
, openssl
, stdenv
, darwin
, lib
}:
# TODO: add wrapProgram to configure path to tex
rustPlatform.buildRustPackage {
  pname = "ict-union-orca";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "sha256-oVycgqvOVwVlEBO+QygTczuVCOwMn51HtjXfX66FFTs=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}
