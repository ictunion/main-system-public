{
  pkg-config,
  openssl,
  stdenv,
  callPackage,
  texlive,
  makeWrapper,
  nix-gitignore,
  buildFeatures ? [ ],
  rustPlatform,
  craneLib,
  system,
  lib,
  darwin,
  advisory-db,
}:
let
  tex = import ./latex { inherit texlive; };
  src = nix-gitignore.gitignoreSource [] ./.;
  nativeBuildInputs = [
    openssl
    pkg-config
    makeWrapper
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

  orca-audit = craneLib.cargoAudit {
    inherit src advisory-db;
  };

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  postInstall = ''
    wrapProgram "$out/bin/orca" --suffix PATH : "${tex}/bin"
  '';
in {
  orca = craneLib.buildPackage {
    inherit cargoArtifacts src buildInputs postInstall nativeBuildInputs;
    cargoExtraArgs =
      if buildFeatures == []
      then ""
      else ''--features "${lib.strings.concatStringsSep " " buildFeatures}"'';
  };

  orca-min = rustPlatform.buildRustPackage {
    inherit src buildInputs postInstall nativeBuildInputs buildFeatures;
    pname = "ict-union-orca-min";
    version = "0.1.0";
    cargoLock = {
      lockFile = ./Cargo.lock;
      allowBuiltinFetchGit = true;
    };
  };

  inherit orca-clippy;
  inherit orca-fmt;
  inherit orca-audit;
}
