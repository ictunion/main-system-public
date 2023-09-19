module type OpaqueString = {
  type t

  let toString: t => string
  let decode: Json.Decode.t<t>
}

module MakeOpaqueString = (): OpaqueString => {
  type t = string

  let toString = uuid => uuid
  let decode = Json.Decode.string
}

module Uuid = MakeOpaqueString()

module Email = MakeOpaqueString()

// TODO: This should be using closed variant type
module Local = MakeOpaqueString()
