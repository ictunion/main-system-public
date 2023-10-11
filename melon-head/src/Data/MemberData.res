open Data

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
