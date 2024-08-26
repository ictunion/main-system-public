open Data

type orcaRole =
  | UnknownOrcaRole(string)
  | ListApplications
  | ViewApplication
  | ResolveApplications
  | ListMembers
  | ViewMember
  | ManageMembers
  | ListWorkplaces
  | ManageWorkplaces
  | SuperPowers

let showOrcaRole = (r: orcaRole): string =>
  switch r {
  | UnknownOrcaRole(str) => str
  | ListApplications => "list-applications"
  | ViewApplication => "view-application"
  | ResolveApplications => "resolve-applications"
  | ListMembers => "list-members"
  | ViewMember => "view-member"
  | ManageMembers => "manage-members"
  | ListWorkplaces => "list-workplaces"
  | ManageWorkplaces => "manage-workplaces"
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
  name: option<string>,
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

type user = {
  id: Data.Uuid.t,
  email: Email.t,
  firstName: option<string>,
  lastName: option<string>,
}

module Decode = {
  open Json.Decode

  let orcaRole = string->map((. str) => {
    switch str {
    | "list-applications" => ListApplications
    | "view-application" => ViewApplication
    | "resolve-applications" => ResolveApplications
    | "list-members" => ListMembers
    | "view-member" => ViewMember
    | "manage-members" => ManageMembers
    | "list-workplaces" => ListWorkplaces
    | "manage-workplaces" => ManageWorkplaces
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
    field.optional(.
      "orca",
      object(field => field.required(. "roles", array(orcaRole))),
    )->Belt.Option.getWithDefault([])
  )

  let realmRoles = object(field =>
    field.optional(.
      "realm-management",
      object(field => field.required(. "roles", array(realmRole))),
    )->Belt.Option.getWithDefault([])
  )

  let tokenClaims = object(field => {
    sub: field.required(. "sub", Data.Uuid.decode),
    email: field.required(. "email", Email.decode),
    name: field.required(. "name", option(string)),
    orcaRoles: field.required(. "resource_access", orcaRoles),
    realmRoles: field.required(. "resource_access", realmRoles),
  })

  let session = object(field => {
    tokenClaims: field.required(. "token_claims", tokenClaims),
    memberId: field.required(. "member_id", option(Data.Uuid.decode)),
  })

  let user = object(field => {
    id: field.required(. "id", Data.Uuid.decode),
    email: field.required(. "email", Email.decode),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
  })
}
