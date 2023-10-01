type error =
  | Empty
  | NetworkError
  | Timeout
  | DecodingError(string, Js.Json.t)

let showError = (err: error): string => {
  switch err {
  | Empty => "Expected JSON value but response was empty"
  | NetworkError => "Network error occured"
  | Timeout => "Request timeout"
  | DecodingError(err, json) =>
    "Error decoding data: " ++ err ++ ". Response: " ++ Js.Json.stringify(json)
  }
}

let fromRequestError = error => {
  switch error {
  | #NetworkRequestFailed => NetworkError
  | #Timeout => Timeout
  }
}

let emptyToError = ({Request.response: response}) => {
  switch response {
  | None => Error(Empty)
  | Some(value) => Ok(value)
  }
}

let mapDecodingError = (res: result<'a, string>, response): result<'a, error> => {
  switch res {
  | Error(str) => Error(DecodingError(str, response))
  | Ok(value) => Ok(value)
  }
}

type t = {
  host: string,
  keycloak: Keycloak.t,
}

let make = (~config: Config.t, ~keycloak: Keycloak.t): t => {
  {
    host: config.apiUrl,
    keycloak,
  }
}

let makeHeaders = api => {
  Js.Dict.fromList(list{
    ("Authorization", "Bearer " ++ api.keycloak->Keycloak.getToken),
    ("Accept", "application/json"),
    ("Content-Type", "application/json"),
  })
}

let fromRequest = (future, decoder) => {
  future
  ->Future.mapError(~propagateCancel=true, fromRequestError)
  ->Future.mapResult(~propagateCancel=true, emptyToError)
  ->Future.mapResult(~propagateCancel=true, res => res->Json.decode(decoder)->mapDecodingError(res))
}

type webData<'a> = RemoteData.t<'a, error>

let getJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>): Future.t<result<'a, error>> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    (),
    ~headers=makeHeaders(api),
  )->fromRequest(decoder)
}

let deleteJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>): Future.t<
  result<'a, error>,
> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#DELETE,
    (),
    ~headers=makeHeaders(api),
  )->fromRequest(decoder)
}

let postJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>, ~body: Js.Json.t): Future.t<
  result<'a, error>,
> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#POST,
    ~headers=makeHeaders(api),
    ~body=Js.Json.stringify(body),
  )
  ->fromRequest(decoder)
}

type status = {
  httpStatus: int,
  httpMessage: string,
  authorizationConnected: bool,
  databaseConnected: bool,
}

module Decode = {
  open Json.Decode

  let status = object(field => {
    httpStatus: field.required(. "http_status", int),
    httpMessage: field.required(. "http_message", string),
    authorizationConnected: field.required(. "authorization_connected", bool),
    databaseConnected: field.required(. "database_connected", bool),
  })
}
