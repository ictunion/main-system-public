with (import <nixpkgs> {});
mkShell {
  name = "ict-administration-panel";
  buildInputs = [
    nodejs-18_x
  ];
}
