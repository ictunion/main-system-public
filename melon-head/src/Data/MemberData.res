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
  companyNames: array<option<string>>,
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

type detail = {
  id: Uuid.t,
  memberNumber: int,
  firstName: option<string>,
  lastName: option<string>,
  dateOfBirth: option<Js.Date.t>,
  email: option<Email.t>,
  phoneNumber: option<PhoneNumber.t>,
  address: option<string>,
  city: option<string>,
  postalCode: option<string>,
  language: option<string>,
  applicationId: option<Uuid.t>,
  leftAt: option<Js.Date.t>,
  onboardingFinishAt: option<Js.Date.t>,
  createdAt: Js.Date.t,
}

type occupation = {
  id: Uuid.t,
  companyName: option<string>,
  position: option<string>,
  createdAt: Js.Date.t,
}

type status =
  | NewMember
  | CurrentMember
  | PastMember

let getStatus = detail => {
  switch detail.leftAt {
  | Some(_) => PastMember
  | None =>
    switch detail.onboardingFinishAt {
    | Some(_) => CurrentMember
    | None => NewMember
    }
  }
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
    companyNames: field.required(. "company_names", array(option(string))),
    createdAt: field.required(. "created_at", date),
  })

  let detail = object(field => {
    id: field.required(. "id", Uuid.decode),
    memberNumber: field.required(. "member_number", int),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    dateOfBirth: field.required(. "date_of_birth", option(date)),
    email: field.required(. "email", option(Email.decode)),
    phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
    address: field.required(. "address", option(string)),
    city: field.required(. "city", option(string)),
    postalCode: field.required(. "postal_code", option(string)),
    language: field.required(. "language", option(string)),
    applicationId: field.required(. "application_id", option(Uuid.decode)),
    leftAt: field.required(. "left_at", option(date)),
    onboardingFinishAt: field.required(. "onboarding_finished_at", option(date)),
    createdAt: field.required(. "created_at", date),
  })

  let occupation = object(field => {
    id: field.required(. "id", Uuid.decode),
    companyName: field.required(. "company_name", option(string)),
    position: field.required(. "position", option(string)),
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

  let newMember = (newMember: newMember) =>
    object([
      ("member_number", option(int, newMember.memberNumber->Option.flatMap(Int.fromString))),
      ("first_name", strOption(newMember.firstName)),
      ("last_name", strOption(newMember.lastName)),
      ("email", strOption(newMember.email)),
      ("phone_number", strOption(newMember.phoneNumber)),
      ("date_of_birth", option(Data.Encode.day, newMember.dateOfBirth)),
      ("address", strOption(newMember.address)),
      ("city", strOption(newMember.city)),
      ("postal_code", strOption(newMember.city)),
      ("language", string(newMember.language)),
    ])
}
