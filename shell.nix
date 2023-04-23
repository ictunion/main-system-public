{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  name = "itc-union-main-system-shell";
  buildInputs = [ refinery-cli postgresql_15 gnumake docker ];
}
