{ stdenv
, refinery-cli
, writeScriptBin
}:
let
  migrations = stdenv.mkDerivation {
    name = "gray-whale-migrations";
    src = ./migrations;
    dontBuild = true;
    installPhase = ''
      mkdir -p $out/var/migrations
      cp $src/*.sql $out/var/migrations
    '';
  };
in
writeScriptBin "gray-whale-migrate" ''
    ${refinery-cli}/bin/refinery $@ -p ${migrations}/var/migrations
''
