open Data
open Belt

type summary = {
  id: Uuid.t,
  memberNumber: int,
  firstName: option<string>,
  lastName: option<string>,
  email: option<Email.t>,
  phoneNumber: option<PhoneNumber.t>,
  city: option<string>,
  leftAt: option<Js.Date.t>,
  createdAt: Js.Date.t,
}

type newMember = {
  memberNumber: option<string>, // Not really a member number until we validate the value
  firstName: string,
  lastName: string,
  // this should be able to hold invalid state as well so it's not email type
  email: string,
  // this should be able to hold invalid state as well so it's not phoneNumber type
  phoneNumber: string,
  dateOfBirth: option<Js.Date.t>,
  address: string,
  city: string,
  postalCode: string,
  language: string,
}

module Decode = {
  open Json.Decode
  let summary = object(field => {
    id: field.required(. "id", Uuid.decode),
    memberNumber: field.required(. "member_number", int),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    email: field.required(. "email", option(Email.decode)),
    phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
    city: field.required(. "city", option(string)),
    leftAt: field.required(. "left_at", option(date)),
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

  let newMember = newMember =>
    object([
      ("member_number", option(int, newMember.memberNumber->Option.flatMap(Int.fromString))),
      ("first_name", strOption(newMember.firstName)),
      ("last_name", strOption(newMember.lastName)),
      ("email", strOption(newMember.email)),
      ("phone_number", strOption(newMember.phoneNumber)),
      ("date_of_birth", option(date, newMember.dateOfBirth)),
      ("address", strOption(newMember.address)),
      ("city", strOption(newMember.city)),
      ("postal_code", strOption(newMember.city)),
      ("language", string(newMember.language)),
    ])
}
