open Data

type status =
  | Processing
  | Unverified
  | Accepted
  | Rejected
  | Invalid

type detail = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  dateOfBirth: option<Date.t>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  address: option<string>,
  postalCode: option<string>,
  occupation: option<string>,
  companyName: option<string>,
  verificationSentAt: option<Date.t>,
  verifiedAt: option<Date.t>,
  registrationIp: option<string>,
  registrationLocal: Local.t,
  registrationUserAgent: option<string>,
  registrationSource: option<string>,
  rejectedAt: option<Date.t>,
  acceptedAt: option<Date.t>,
  invalidatedAt: option<Date.t>,
  createdAt: Date.t,
}

let getStatus = (detail: detail): status => {
  switch detail.acceptedAt {
  | Some(_) => Accepted
  | None =>
    switch detail.rejectedAt {
    | Some(_) => Rejected
    | None =>
      switch detail.invalidatedAt {
      | Some(_) => Invalid
      | None =>
        switch detail.verifiedAt {
        | Some(_) => Processing
        | None => Unverified
        }
      }
    }
  }
}

type summary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  approvedByRepsAt: option<Date.t>,
}

type acceptedSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  acceptedAt: Date.t,
  memberId: Uuid.t,
  approvedByRepsAt: option<Date.t>,
}

type rejectedSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  rejectedAt: Date.t,
  approvedByRepsAt: option<Date.t>,
}

type invalidSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  invalidatedAt: Date.t,
  approvedByRepsAt: option<Date.t>,
}

type processingSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  verifiedAt: Date.t,
  approvedByRepsAt: option<Date.t>,
}

type unverifiedSummary = {
  id: Uuid.t,
  email: option<Email.t>,
  firstName: option<string>,
  lastName: option<string>,
  phoneNumber: option<PhoneNumber.t>,
  note: option<string>,
  city: option<string>,
  companyName: option<string>,
  registrationLocal: Local.t,
  createdAt: Date.t,
  verificationSentAt: option<Date.t>,
  approvedByRepsAt: option<Date.t>,
}

type workplace = {
  workplace: WorkplaceData.summary,
  approvedByRepsAt: option<Date.t>,
}

module Decode = {
  open Json.Decode

  let detail = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    dateOfBirth: field.required("date_of_birth", option(Data.Decode.day)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    address: field.required("address", option(string)),
    postalCode: field.required("postal_code", option(string)),
    occupation: field.required("occupation", option(string)),
    companyName: field.required("company_name", option(string)),
    verificationSentAt: field.required("verification_sent_at", option(date)),
    verifiedAt: field.required("confirmed_at", option(date)),
    registrationIp: field.required("registration_ip", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    registrationUserAgent: field.required("registration_user_agent", option(string)),
    registrationSource: field.required("registration_source", option(string)),
    rejectedAt: field.required("rejected_at", option(date)),
    invalidatedAt: field.required("invalidated_at", option(date)),
    acceptedAt: field.required("accepted_at", option(date)),
    createdAt: field.required("created_at", date),
  })

  let summary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let acceptedSummary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    acceptedAt: field.required("accepted_at", date),
    memberId: field.required("member_id", Uuid.decode),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let rejectedSummary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    rejectedAt: field.required("rejected_at", date),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let invalidSummary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    invalidatedAt: field.required("invalidated_at", date),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let processingSummary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    verifiedAt: field.required("confirmed_at", date),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let unverifiedSummary = object(field => {
    id: field.required("id", Uuid.decode),
    email: field.required("email", option(Email.decode)),
    firstName: field.required("first_name", option(string)),
    lastName: field.required("last_name", option(string)),
    phoneNumber: field.required("phone_number", option(PhoneNumber.decode)),
    note: field.required("note", option(string)),
    city: field.required("city", option(string)),
    companyName: field.required("company_name", option(string)),
    registrationLocal: field.required("registration_local", Local.decode),
    createdAt: field.required("created_at", date),
    verificationSentAt: field.required("verification_sent_at", option(date)),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })

  let workplace = object(field => {
    workplace: field.required("workplace", WorkplaceData.Decode.summary),
    approvedByRepsAt: field.required("approved_by_reps_at", option(date)),
  })
}
