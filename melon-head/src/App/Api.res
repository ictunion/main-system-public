type apiError = {
  code: int,
  description: string,
  reason: string, // This could be eventually turned into enum/variant
}

type error =
  | Empty
  | NetworkError
  | Timeout
  | DecodingError(string, Js.Json.t)
  | ApiError(apiError)

type status = {
  httpStatus: int,
  httpMessage: string,
  authorizationConnected: bool,
  databaseConnected: bool,
  proxySupportEnabled: bool,
}

type acceptedResponse = {
  status: int,
  message: string,
}

module Decode = {
  open Json.Decode

  let status = object(field => {
    httpStatus: field.required(. "http_status", int),
    httpMessage: field.required(. "http_message", string),
    authorizationConnected: field.required(. "authorization_connected", bool),
    databaseConnected: field.required(. "database_connected", bool),
    proxySupportEnabled: field.required(. "proxy_support_enabled", bool),
  })

  let apiError = object(field => {
    field.required(.
      "error",
      object(field => {
        code: field.required(. "code", int),
        description: field.required(. "description", string),
        reason: field.required(. "reason", string),
      }),
    )
  })

  let acceptedResponse = object(field => {
    status: field.required(. "status", int),
    message: field.required(. "message", string),
  })
}

let showError = (err: error): string => {
  switch err {
  | Empty => "Expected JSON value but response was empty"
  | NetworkError => "Network error occured"
  | Timeout => "Request timeout"
  | DecodingError(err, json) =>
    "Error decoding data: " ++ err ++ ". Response: " ++ Js.Json.stringify(json)
  | ApiError(apiError) => apiError.description
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
  | Ok(value) => Ok(value)
  | Error(str) =>
    switch Json.decode(response, Decode.apiError) {
    | Ok(apiError) => Error(ApiError(apiError))
    | Error(_) => Error(DecodingError(str, response))
    }
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

let makeJsonHeaders = api => {
  Js.Dict.fromList(list{
    ("Authorization", "Bearer " ++ api.keycloak->Keycloak.getToken),
    ("Accept", "application/json"),
    ("Content-Type", "application/json"),
  })
}

let makeTextHeaders = api => {
  Js.Dict.fromList(list{
    ("Authorization", "Bearer " ++ api.keycloak->Keycloak.getToken),
    ("Accept", "text/csv"),
  })
}

let fromJsonRequest = (future, decoder) => {
  future
  ->Future.mapError(~propagateCancel=true, fromRequestError)
  ->Future.mapResult(~propagateCancel=true, emptyToError)
  ->Future.mapResult(~propagateCancel=true, res => res->Json.decode(decoder)->mapDecodingError(res))
}

let fromTextRequest = future => {
  future
  ->Future.mapError(~propagateCancel=true, fromRequestError)
  ->Future.mapResult(~propagateCancel=true, emptyToError)
}

type webData<'a> = RemoteData.t<'a, error>

let getText = (api: t, ~path: string): Future.t<result<string, error>> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Text,
    (),
    ~headers=makeTextHeaders(api),
  )->fromTextRequest
}

let getJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>): Future.t<result<'a, error>> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    (),
    ~headers=makeJsonHeaders(api),
  )->fromJsonRequest(decoder)
}

let deleteJson = (
  api: t,
  ~path: string,
  ~decoder: Json.Decode.t<'a>,
  ~body: option<Js.Json.t>,
): Future.t<result<'a, error>> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#DELETE,
    (),
    ~headers=makeJsonHeaders(api),
    ~body=?body->Belt.Option.map(Js.Json.stringify),
  )->fromJsonRequest(decoder)
}

let postJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>, ~body: Js.Json.t): Future.t<
  result<'a, error>,
> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#POST,
    ~headers=makeJsonHeaders(api),
    ~body=Js.Json.stringify(body),
  )
  ->fromJsonRequest(decoder)
}

let patchJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>, ~body: Js.Json.t): Future.t<
  result<'a, error>,
> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#PATCH,
    ~headers=makeJsonHeaders(api),
    ~body=Js.Json.stringify(body),
  )
  ->fromJsonRequest(decoder)
}
