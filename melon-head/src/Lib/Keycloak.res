module Exn = {
  type keycloak
  @get external token: keycloak => string = "token"
  @send external logout: keycloak => unit = "logout"
}

type t = Exn.keycloak

let getToken = Exn.token
let logout = Exn.logout
