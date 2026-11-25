// Loads the Orca/bank API response bodies in test/fixtures/ (Vitest runs from melon-head/).
@module("node:fs") external readFileSync: (string, string) => string = "readFileSync"

let load = (name: string): JSON.t =>
  JSON.parseOrThrow(readFileSync("test/fixtures/" ++ name ++ ".json", "utf8"))

let decode = (name: string, decoder: Json.Decode.t<'a>): 'a =>
  switch Json.decode(load(name), decoder) {
  | Ok(value) => value
  | Error(msg) => JsError.throwWithMessage(name ++ ": " ++ msg)
  }

// The fixture with one top-level field changed (Some) or removed (None).
let withField = (name: string, key: string, value: option<JSON.t>): JSON.t =>
  switch load(name) {
  | Object(fields) =>
    let fields = fields->Dict.copy
    switch value {
    | Some(v) => fields->Dict.set(key, v)
    | None => fields->Dict.delete(key)
    }
    Object(fields)
  | _ => JsError.throwWithMessage(name ++ " is not a JSON object")
  }
