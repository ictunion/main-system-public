{
  description = "ITC Union official website";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    flake-utils.url = github:numtide/flake-utils;
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [];
      };
      tex = pkgs.texlive.combine {
        inherit (pkgs.texlive)
          scheme-small
          datetime
          fmtcount;
      };
      buildInputs = with pkgs; [
        tex
        rustup
        openssl
        pkg-config
      ];
    in
    {
      devShell = with pkgs;
        mkShell {
          name = "ict-union-orca-dev-env";
          inherit buildInputs;
          shellHook = ''
            rustup install stable
            rustup override set stable
          '';
        };
      defaultPackage = pkgs.callPackage ./default.nix {};
  });
}
