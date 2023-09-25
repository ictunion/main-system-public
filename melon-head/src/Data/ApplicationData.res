open Data

type status =
  | Processing
  | Unverified
  | Accepted
  | Rejected

type detail = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  dateOfBirth: option<Js.Date.t>,
  phoneNumber: option<PhoneNumber.t>,
  city: option<string>,
  address: option<string>,
  postalCode: option<string>,
  occupation: option<string>,
  companyName: option<string>,
  verificationSentAt: option<Js.Date.t>,
  verifiedAt: option<Js.Date.t>,
  registrationIp: option<string>,
  registrationLocal: Local.t,
  registrationUserAgent: option<string>,
  registrationSource: option<string>,
  rejectedAt: option<Js.Date.t>,
  acceptedAt: option<Js.Date.t>,
  createdAt: Js.Date.t,
}

let getStatus = (detail: detail): status => {
  switch detail.acceptedAt {
  | Some(_) => Accepted
  | None =>
    switch detail.rejectedAt {
    | Some(_) => Rejected
    | None =>
      switch detail.verifiedAt {
      | Some(_) => Processing
      | None => Unverified
      }
    }
  }
}

type processingSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Js.Date.t,
  verifiedAt: Js.Date.t,
}

type unverifiedSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Js.Date.t,
  verificationSentAt: option<Js.Date.t>,
}

module Decode = {
  open Json.Decode

  let detail = object(field => {
    id: field.required(. "id", Uuid.decode),
    email: field.required(. "email", option(Email.decode)),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    dateOfBirth: field.required(. "date_of_birth", option(date)),
    phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
    city: field.required(. "city", option(string)),
    address: field.required(. "address", option(string)),
    postalCode: field.required(. "postal_code", option(string)),
    occupation: field.required(. "occupation", option(string)),
    companyName: field.required(. "company_name", option(string)),
    verificationSentAt: field.required(. "verification_sent_at", option(date)),
    verifiedAt: field.required(. "confirmed_at", option(date)),
    registrationIp: field.required(. "registration_ip", option(string)),
    registrationLocal: field.required(. "registration_local", Local.decode),
    registrationUserAgent: field.required(. "registration_user_agent", option(string)),
    registrationSource: field.required(. "registration_source", option(string)),
    rejectedAt: field.required(. "rejected_at", option(date)),
    acceptedAt: field.required(. "accepted_at", option(date)),
    createdAt: field.required(. "created_at", date),
  })

  let processingSummary = object(field => {
    id: field.required(. "id", Uuid.decode),
    email: field.required(. "email", option(Email.decode)),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
    city: field.required(. "city", option(string)),
    companyName: field.required(. "company_name", option(string)),
    registrationLocal: field.required(. "registration_local", Local.decode),
    createdAt: field.required(. "created_at", date),
    verifiedAt: field.required(. "confirmed_at", date),
  })

  let unverifiedSummary = object(field => {
    id: field.required(. "id", Uuid.decode),
    email: field.required(. "email", option(Email.decode)),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
    city: field.required(. "city", option(string)),
    companyName: field.required(. "company_name", option(string)),
    registrationLocal: field.required(. "registration_local", Local.decode),
    createdAt: field.required(. "created_at", date),
    verificationSentAt: field.required(. "verification_sent_at", option(date)),
  })
}
