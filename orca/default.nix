{ pkg-config
, openssl
, stdenv
, callPackage
, texlive
, makeWrapper
, nix-gitignore
, buildFeatures ? []
, craneLib
, system
, lib
, darwin
}:
let
  tex = import ./latex { inherit texlive; };
  src = nix-gitignore.gitignoreSource [] ./.;
  nativeBuildInputs = [
    openssl
    pkg-config
  ];

  # Build *just* the cargo dependencies, so we can reuse
  # all of that work (e.g. via cachix) when running in CI
  cargoArtifacts = craneLib.buildDepsOnly {
    inherit src nativeBuildInputs;
  };

  # Run clippy (and deny all warnings) on the crate source,
  # resuing the dependency artifacts (e.g. from build scripts or
  # proc-macros) from above.
  #
  # Note that this is done as a separate derivation so it
  # does not impact building just the crate by itself.
  orca-clippy = craneLib.cargoClippy {
    inherit cargoArtifacts src nativeBuildInputs;
    cargoClippyExtraArgs = "-- --deny warnings";
  };

  orca-fmt = craneLib.cargoFmt {
    inherit src;
  };
in
rec {
  orca = craneLib.buildPackage {
    inherit cargoArtifacts src;
    cargoExtraArgs =
      if buildFeatures == []
      then ""
      else ''--features "${lib.strings.concatStringsSep " " buildFeatures}"'';
    nativeBuildInputs = nativeBuildInputs ++ [ makeWrapper ];
    buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
    ];
    postInstall = ''
      wrapProgram "$out/bin/orca" --suffix PATH : "${tex}/bin"
  '';
  };

  inherit orca-clippy;
  inherit orca-fmt;
}
