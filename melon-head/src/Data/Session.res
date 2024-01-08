open Data

type orcaRole =
  | UnknownOrcaRole(string)
  | ListApplications
  | ViewApplication
  | ResolveApplications
  | ListMembers
  | ManageMembers
  | SuperPowers

let showOrcaRole = (r: orcaRole): string =>
  switch r {
  | UnknownOrcaRole(str) => str
  | ListApplications => "list-applications"
  | ViewApplication => "view-application"
  | ResolveApplications => "resolve-applications"
  | ListMembers => "list-members"
  | ManageMembers => "manage-members"
  | SuperPowers => "super-powers"
  }

type realmRole =
  | UnknownRealmRole(string)
  | ManageUsers

let showRealmRole = (r: realmRole): string =>
  switch r {
  | UnknownRealmRole(str) => str
  | ManageUsers => "manage-users"
  }

type tokenClaims = {
  sub: Data.Uuid.t,
  email: Email.t,
  name: string,
  orcaRoles: array<orcaRole>,
  realmRoles: array<realmRole>,
}

type t = {
  tokenClaims: tokenClaims,
  memberId: option<Data.Uuid.t>,
}

let hasRole = (session, ~role: orcaRole): bool => {
  open Belt

  let allRoles = session.tokenClaims.orcaRoles
  Array.some(allRoles, r => r == role)
}

let hasRealmRole = (session, ~role: realmRole): bool => {
  open Belt
  let allRoles = session.tokenClaims.realmRoles
  Array.some(allRoles, r => r == role)
}

module Decode = {
  open Json.Decode

  let orcaRole = string->map((. str) => {
    switch str {
    | "list-applications" => ListApplications
    | "view-application" => ViewApplication
    | "resolve-applications" => ResolveApplications
    | "list-members" => ListMembers
    | "manage-members" => ManageMembers
    | "super-powers" => SuperPowers
    | _ => UnknownOrcaRole(str)
    }
  })

  let realmRole = string->map((. str) => {
    switch str {
    | "manage-users" => ManageUsers
    | _ => UnknownRealmRole(str)
    }
  })

  let orcaRoles = object(field =>
    field.required(. "orca", object(field => field.required(. "roles", array(orcaRole))))
  )

  let realmRoles = object(field =>
    field.required(.
      "realm-management",
      object(field => field.required(. "roles", array(realmRole))),
    )
  )

  let tokenClaims = object(field => {
    sub: field.required(. "sub", Data.Uuid.decode),
    email: field.required(. "email", Email.decode),
    name: field.required(. "name", string),
    orcaRoles: field.required(. "resource_access", orcaRoles),
    realmRoles: field.required(. "resource_access", realmRoles),
  })

  let session = object(field => {
    tokenClaims: field.required(. "token_claims", tokenClaims),
    memberId: field.required(. "member_id", option(Data.Uuid.decode)),
  })
}
