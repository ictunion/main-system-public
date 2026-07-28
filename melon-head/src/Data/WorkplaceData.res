open Data

type newWorkplace = {
  name: string,
  email: string,
  keycloakGroupId: string,
  keycloakExecutiveGroupId: string,
  newsletterId: string,
}

type workplaceMember = {memberId: string}

type summary = {
  id: Uuid.t,
  name: string,
  email: Email.t,
  keycloakGroupId: Uuid.t,
  keycloakExecutiveGroupId: option<Uuid.t>,
  newsletterId: option<int>,
  memberCount: int,
  createdAt: Js.Date.t,
  establishedAt: option<Js.Date.t>,
  announcedAt: option<Js.Date.t>,
  cancelledAt: option<Js.Date.t>,
}

type status = Initial | Established | Announced | Cancelled

let getStatus = (d: summary): status =>
  if d.cancelledAt->Belt.Option.isSome {
    Cancelled
  } else if d.announcedAt->Belt.Option.isSome {
    Announced
  } else if d.establishedAt->Belt.Option.isSome {
    Established
  } else {
    Initial
  }

module Decode = {
  open Json.Decode

  let summary = object(field => {
    id: field.required(. "id", Uuid.decode),
    name: field.required(. "name", string),
    email: field.required(. "email", Email.decode),
    keycloakGroupId: field.required(. "keycloak_group_id", Uuid.decode),
    keycloakExecutiveGroupId: field.required(. "keycloak_executive_group_id", option(Uuid.decode)),
    newsletterId: field.required(. "newsletter_id", option(int)),
    memberCount: field.required(. "member_count", int),
    createdAt: field.required(. "created_at", date),
    establishedAt: field.required(. "established_at", option(date)),
    announcedAt: field.required(. "announced_at", option(date)),
    cancelledAt: field.required(. "cancelled_at", option(date)),
  })
}

module Encode = {
  open Json.Encode

  let strOption = str => {
    if str == "" {
      null
    } else {
      string(str)
    }
  }

  let newWorkplace = (newWorkplace: newWorkplace) =>
    object([
      ("name", strOption(newWorkplace.name)),
      ("email", strOption(newWorkplace.email)),
      ("keycloak_group_id", strOption(newWorkplace.keycloakGroupId)),
      ("keycloak_executive_group_id", strOption(newWorkplace.keycloakExecutiveGroupId)),
      ("newsletter_id", option(int, newWorkplace.newsletterId->Belt.Int.fromString)),
    ])

  let workplaceMember = (workplaceMember: workplaceMember) =>
    object([("member_id", strOption(workplaceMember.memberId))])
}
