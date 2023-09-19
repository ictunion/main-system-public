type orcaRole =
  | UnknownOrcaRole(string)
  | ListApplications

let showOrcaRole = (r: orcaRole): string =>
  switch r {
  | UnknownOrcaRole(str) => str
  | ListApplications => "list-applications"
  }

type tokenClaims = {
  sub: string, // UUID identifying keycloak user
  realmRoles: array<string>,
  orcaRoles: array<orcaRole>,
}

type t = {tokenClaims: tokenClaims}

let hasRole = (session, ~role: orcaRole): bool => {
  open Belt

  let allRoles = session.tokenClaims.orcaRoles
  Array.some(allRoles, r => r == role)
}

module Decode = {
  open Json.Decode

  let orcaRole = string->map((. str) => {
    switch str {
    | "list-applications" => ListApplications
    | _ => UnknownOrcaRole(str)
    }
  })

  let realmRoles = object(field => field.required(. "roles", array(string)))

  let orcaRoles = object(field =>
    field.required(. "orca", object(field => field.required(. "roles", array(orcaRole))))
  )

  let tokenClaims = object(field => {
    sub: field.required(. "sub", string),
    realmRoles: field.required(. "realm_access", realmRoles),
    orcaRoles: field.required(. "resource_access", orcaRoles),
  })

  let session = object(field => {
    tokenClaims: field.required(. "token_claims", tokenClaims),
  })
}
