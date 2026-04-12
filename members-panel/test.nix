with (import <nixpkgs> {});
callPackage ./. {
  nodejs = nodejs_24;
  config = builtins.fromJSON (builtins.readFile ./config.example.json);
}
