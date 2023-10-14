module type OpaqueString = {
  type t

  let toString: t => string
  let unsafeFromString: string => t
  let decode: Json.Decode.t<t>
  let encode: t => Js.Json.t
}

module MakeOpaqueString = (): OpaqueString => {
  type t = string

  let toString = uuid => uuid
  let unsafeFromString = str => str
  let decode = Json.Decode.string
  let encode = Json.Encode.string
}

module Uuid = MakeOpaqueString()

module Email = MakeOpaqueString()

module PhoneNumber = MakeOpaqueString()

// TODO: This should be using closed variant type
module Local = MakeOpaqueString()

type file = {
  id: Uuid.t,
  name: string,
  fileType: string,
  createdAt: Js.Date.t,
}

module Decode = {
  open Json.Decode

  let file = object(field => {
    id: field.required(. "id", Uuid.decode),
    name: field.required(. "name", string),
    fileType: field.required(. "file_type", string),
    createdAt: field.required(. "created_at", date),
  })
}

module Encode = {
  open Json.Encode

  let day = (date: Js.Date.t): Js.Json.t => {
    let str = Js.Date.toJSONUnsafe(date)->Js.String.slice(~from=0, ~to_=10)
    string(str)
  }
}
