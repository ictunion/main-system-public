open Data

type newWorkplace = {
  name: string,
  email: string,
}

type workplaceMember = {memberId: string}

type summary = {
  id: Uuid.t,
  name: string,
  email: Email.t,
  createdAt: Js.Date.t,
}

module Decode = {
  open Json.Decode

  let summary = object(field => {
    id: field.required(. "id", Uuid.decode),
    name: field.required(. "name", string),
    email: field.required(. "email", Email.decode),
    createdAt: field.required(. "created_at", date),
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
    object([("name", strOption(newWorkplace.name)), ("email", strOption(newWorkplace.email))])

  let workplaceMember = (workplaceMember: workplaceMember) =>
    object([("member_id", strOption(workplaceMember.memberId))])
}
