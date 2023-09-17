{ pkgs ? import <nixpkgs> {} }:
with pkgs;
callPackage ./. {
  nodejs = nodejs-18_x;
  config = builtins.fromJSON (builtins.readFile ./config.example.json);
}
