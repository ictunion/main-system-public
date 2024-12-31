{
  description = "ITC Union application and member management system";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    flake-utils.url = github:numtide/flake-utils;
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };

    orca-stuff =
      let
        tex = with pkgs; import ./orca/latex { inherit texlive; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ]; # RustRover wants stdlib sources
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        buildInputs = with pkgs; [
          tex
          openssl
          pkg-config
          craneLib.rustc
          craneLib.cargo
        ];

        orcaPkgs = pkgs.callPackage ./orca {
          inherit craneLib;
        };
      in {
        devShell = with pkgs;
          mkShell {
            name = "ict-union-orca-dev-env";
            inherit buildInputs;
            OSFONTDIR = "${pkgs.ibm-plex}/share/fonts/opentype";
          };
        package = orcaPkgs.orca;
        checks = {
          inherit (orcaPkgs) orca orca-clippy orca-fmt;
        };
      };
    gray-whale-stuff =
      let
        buildInputs = with pkgs; [
          refinery-cli
        ];
      in
      {
        devShell = with pkgs;
          mkShell {
            name = "ict-union-gray-whale-dev";
            inherit buildInputs;
          };
        package = pkgs.callPackage ./gray-whale {};
      };
  in
    {
      devShells = {
        orca = orca-stuff.devShell;
        gray-whale = gray-whale-stuff.devShell;
        default = pkgs.mkShell {
          packages = with pkgs; [ gnumake postgresql_15 ];
        };
      };

      packages = {
        orca = orca-stuff.package;
        gray-whale = gray-whale-stuff.package;
      };

      checks = orca-stuff.checks;
    });
}
