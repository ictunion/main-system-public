module Applications = {
  type basic = {
    unverified: int,
    accepted: int,
    rejected: int,
    processing: int,
    invalid: int,
  }

  let all = (b: basic): int => {
    b.unverified + b.accepted + b.rejected + b.processing + b.invalid
  }

  module Decode = {
    open Json.Decode

    let basic = object(field => {
      unverified: field.required(. "unverified", int),
      accepted: field.required(. "accepted", int),
      rejected: field.required(. "rejected", int),
      processing: field.required(. "processing", int),
      invalid: field.required(. "invalid", int),
    })
  }
}

module Members = {
  type basic = {
    new: int,
    current: int,
    past: int,
    workplace: int,
    sectoral: int,
  }

  let all = (b: basic): int => {
    b.new + b.current + b.past
  }

  // Members who have not left -- the denominator for the sectoral/workplace split.
  let active = (b: basic): int => {
    b.new + b.current
  }

  module Decode = {
    open Json.Decode

    let basic = object(field => {
      new: field.required(. "new", int),
      current: field.required(. "current", int),
      past: field.required(. "past", int),
      workplace: field.required(. "workplace", int),
      sectoral: field.required(. "sectoral", int),
    })
  }
}

module Workplaces = {
  type basic = {
    current: int,
    past: int,
  }

  let all = (b: basic): int => {
    b.current + b.past
  }

  module Decode = {
    open Json.Decode

    let basic = object(field => {
      current: field.required(. "current", int),
      past: field.required(. "past", int),
    })
  }
}
