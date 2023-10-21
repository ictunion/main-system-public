{ python3
, nix-gitignore
}:
python3.pkgs.buildPythonApplication rec {
  pname = "ictunion-main-system-acceptance-test";
  version = "0.1.0";
  src = nix-gitignore.gitignoreSource [] ./.;
  propagatedBuildInputs = with python3.pkgs; [
    requests
  ];
  # By default tests are executed, but they need to be invoked differently for this package
  dontUseSetuptoolsCheck = true;
}
