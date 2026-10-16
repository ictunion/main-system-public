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

/* Calendar days ("2026-05-01", `NaiveDate` in Orca) are handled as local
   midnight, so the date picker and `toLocaleDateString` show the same day the
   API sent. Going through `Js.Date.toJSON` would convert to UTC first and shift
   the day back by the UTC offset (e.g. a date of birth picked in Prague would
   be stored one day earlier). */
module Day = {
  let fromString = (str: string): Js.Date.t => {
    switch str->Js.String2.split("-")->Belt.Array.map(Belt.Int.fromString) {
    | [Some(year), Some(month), Some(date)] =>
      Js.Date.makeWithYMD(
        ~year=Belt.Int.toFloat(year),
        ~month=Belt.Int.toFloat(month - 1),
        ~date=Belt.Int.toFloat(date),
        (),
      )
    | _ => Js.Date.fromString(str)
    }
  }

  let toString = (date: Js.Date.t): string => {
    let pad = n => (n < 10 ? "0" : "") ++ Belt.Int.toString(n)
    let year = date->Js.Date.getFullYear->Belt.Int.fromFloat->Belt.Int.toString
    let month = (date->Js.Date.getMonth->Belt.Int.fromFloat + 1)->pad
    let day = date->Js.Date.getDate->Belt.Int.fromFloat->pad
    year ++ "-" ++ month ++ "-" ++ day
  }
}

type file = {
  id: Uuid.t,
  name: string,
  fileType: string,
  createdAt: Js.Date.t,
}

module Decode = {
  open Json.Decode

  let day = string->map((. str) => Day.fromString(str))

  let file = object(field => {
    id: field.required(. "id", Uuid.decode),
    name: field.required(. "name", string),
    fileType: field.required(. "file_type", string),
    createdAt: field.required(. "created_at", date),
  })
}

module Encode = {
  open Json.Encode

  let day = (date: Js.Date.t): Js.Json.t => string(Day.toString(date))
}
