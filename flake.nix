{
  description = "ITC Union official website";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    flake-utils.url = github:numtide/flake-utils;
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, crane }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [];
    };

    orca-stuff =
      let
        tex = with pkgs; import ./orca/latex { inherit texlive; };
        buildInputs = with pkgs; [
          tex
          rustup
          openssl
          pkg-config
        ];

        orcaPkgs = pkgs.callPackage ./orca {
          inherit crane;
        };
      in {
        devShell = with pkgs;
          mkShell {
            name = "ict-union-orca-dev-env";
            inherit buildInputs;
            shellHook = ''
              rustup install stable
              rustup override set stable
            '';
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
      };

      packages = {
        orca = orca-stuff.package;
        gray-whale = gray-whale-stuff.package;
      };

      checks = orca-stuff.checks;
    });
}
