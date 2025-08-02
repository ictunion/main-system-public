{ pkgs ? import <nixpkgs> {} }:
with pkgs;
callPackage ./. {
  nodejs = nodejs_20;
  config = builtins.fromJSON (builtins.readFile ./config.example.json);
}
