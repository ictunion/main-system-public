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
      tex = with pkgs; import ./latex { inherit texlive; };
      buildInputs = with pkgs; [
        tex
        rustup
        openssl
        pkg-config
      ];

      orcaPkgs = pkgs.callPackage ./default.nix {
        inherit crane;
      };
    in rec
    {
      devShells.default = with pkgs;
        mkShell {
          name = "ict-union-orca-dev-env";
          inherit buildInputs;
          shellHook = ''
            rustup install stable
            rustup override set stable
          '';
          OSFONTDIR = "${pkgs.ibm-plex}/share/fonts/opentype";
        };

      defaultPackage = orcaPkgs.orca;

      checks = {
        inherit (orcaPkgs) orca orca-clippy orca-fmt;
      };
    });
}
