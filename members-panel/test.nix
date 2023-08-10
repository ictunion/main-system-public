with (import <nixpkgs> {});
callPackage ./. {
  nodejs = nodejs-18_x;
  config = {
    keycloak_url = "https://keycloak.ictunion.cz";
    keycloak_realm = "members";
    keycloak_client_id = "test-client";
  };
}
