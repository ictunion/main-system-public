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
      defaultPackage = pkgs.callPackage ./default.nix {};
  });
}
