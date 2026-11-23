type apiError = {
  code: int,
  description: string,
  reason: string, // This could be eventually turned into enum/variant
}

type error =
  | Empty
  | NetworkError
  | Timeout
  | DecodingError(string, JSON.t)
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
    httpStatus: field.required("http_status", int),
    httpMessage: field.required("http_message", string),
    authorizationConnected: field.required("authorization_connected", bool),
    databaseConnected: field.required("database_connected", bool),
    proxySupportEnabled: field.required("proxy_support_enabled", bool),
  })

  let apiError = object(field => {
    field.required(
      "error",
      object(field => {
        code: field.required("code", int),
        description: field.required("description", string),
        reason: field.required("reason", string),
      }),
    )
  })

  let acceptedResponse = object(field => {
    status: field.required("status", int),
    message: field.required("message", string),
  })
}

let showError = (err: error): string => {
  switch err {
  | Empty => "Expected JSON value but response was empty"
  | NetworkError => "Network error occured"
  | Timeout => "Request timeout"
  | DecodingError(err, json) =>
    "Error decoding data: " ++ err ++ ". Response: " ++ JSON.stringify(json)
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
  oidc: Oidc.t,
}

// Read the user on every call -- silent renew swaps it without re-rendering,
// so a user captured at render time sends an expired token.
let getToken = (api: t) => {
  Oidc.getUser(api.oidc)->Oidc.User.getToken
}

let make = (~config: Config.t, ~oidc: Oidc.t): t => {
  {
    host: config.apiUrl,
    oidc,
  }
}

let makeBank = (~config: Config.t, ~oidc: Oidc.t): t => {
  {
    // The bank API serves everything under an /api prefix; request paths stay
    // prefix-free (e.g. "/payments/..."). When bank_api_url is unset the host
    // is meaningless -- callers must gate requests on Config.bankEnabled
    // instead of relying on this falling back to a dead URL.
    host: config.bankApiUrl->Option.getOr("") ++ "/api",
    oidc,
  }
}

let makeJsonHeaders = api => {
  Dict.fromArray([
    ("Authorization", "Bearer " ++ getToken(api)),
    ("Accept", "application/json"),
    ("Content-Type", "application/json"),
  ])
}

let makeTextHeaders = api => {
  Dict.fromArray([("Authorization", "Bearer " ++ getToken(api)), ("Accept", "text/csv")])
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
  ~body: option<JSON.t>,
): Future.t<result<'a, error>> => {
  Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#DELETE,
    (),
    ~headers=makeJsonHeaders(api),
    ~body=?body->Option.map(b => JSON.stringify(b)),
  )->fromJsonRequest(decoder)
}

let postJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>, ~body: JSON.t): Future.t<
  result<'a, error>,
> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#POST,
    ~headers=makeJsonHeaders(api),
    ~body=JSON.stringify(body),
  )
  ->fromJsonRequest(decoder)
}

let put = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>): Future.t<result<'a, error>> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#PUT,
    ~headers=makeJsonHeaders(api),
  )
  ->fromJsonRequest(decoder)
}

let patchJson = (api: t, ~path: string, ~decoder: Json.Decode.t<'a>, ~body: JSON.t): Future.t<
  result<'a, error>,
> => {
  ()
  ->Request.make(
    ~url=api.host ++ path,
    ~responseType=Json,
    ~method=#PATCH,
    ~headers=makeJsonHeaders(api),
    ~body=JSON.stringify(body),
  )
  ->fromJsonRequest(decoder)
}
