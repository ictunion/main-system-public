type basic = {
  unverified: int,
  accepted: int,
  rejected: int,
  processing: int,
}

module Decode = {
  open Json.Decode

  let basic = object(field => {
    unverified: field.required(. "unverified", int),
    accepted: field.required(. "accepted", int),
    rejected: field.required(. "rejected", int),
    processing: field.required(. "processing", int),
  })
}
