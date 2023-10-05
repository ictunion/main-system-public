type basic = {
  unverified: int,
  accepted: int,
  rejected: int,
  processing: int,
}

let all = (b: basic): int => {
  b.unverified + b.accepted + b.rejected + b.processing
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
