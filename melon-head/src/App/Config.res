type t = {
  apiUrl: string,
  keycloakUrl: string,
  keycloakRealm: string,
  keycloakClietId: string,
  profileUrl: string,
  keycloakMembersGroupId: option<Data.Uuid.t>,
}

module Decode = {
  open Json.Decode

  let config = object(field => {
    apiUrl: field.required(. "api_url", string),
    keycloakUrl: field.required(. "keycloak_url", string),
    keycloakRealm: field.required(. "keycloak_realm", string),
    keycloakClietId: field.required(. "keycloak_client_id", string),
    profileUrl: field.required(. "profile_url", string),
    keycloakMembersGroupId: field.optional(. "keycloak_members_group_id", Data.Uuid.decode),
  })
}

let make = (json: Js.Json.t): result<t, string> => {
  Json.decode(json, Decode.config)
}

let keycloakAccountLink = ({keycloakUrl, keycloakRealm}: t): string => {
  keycloakUrl ++ "/realms/" ++ keycloakRealm ++ "/account/"
}
