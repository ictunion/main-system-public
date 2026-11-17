module type OpaqueString = {
  type t

  let toString: t => string
  let unsafeFromString: string => t
  let decode: Json.Decode.t<t>
  let encode: t => JSON.t
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
   API sent. Going through `Date.toJSON` would convert to UTC first and shift
   the day back by the UTC offset (e.g. a date of birth picked in Prague would
   be stored one day earlier). */
module Day = {
  let fromString = (str: string): Date.t => {
    switch str->String.split("-")->Array.map(s => Int.fromString(s)) {
    | [Some(year), Some(month), Some(date)] => Date.makeWithYMD(~year, ~month=month - 1, ~day=date)
    | _ => Date.fromString(str)
    }
  }

  let toString = (date: Date.t): string => {
    let pad = n => (n < 10 ? "0" : "") ++ Int.toString(n)
    let year = date->Date.getFullYear->Int.toString
    let month = (date->Date.getMonth + 1)->pad
    let day = date->Date.getDate->pad
    year ++ "-" ++ month ++ "-" ++ day
  }
}

type file = {
  id: Uuid.t,
  name: string,
  fileType: string,
  createdAt: Date.t,
}

module Decode = {
  open Json.Decode

  let day = string->map(str => Day.fromString(str))

  let file = object(field => {
    id: field.required("id", Uuid.decode),
    name: field.required("name", string),
    fileType: field.required("file_type", string),
    createdAt: field.required("created_at", date),
  })
}

module Encode = {
  open Json.Encode

  let day = (date: Date.t): JSON.t => string(Day.toString(date))
}
