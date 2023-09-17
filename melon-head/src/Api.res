type error =
  | Empty
  | NetworkError
  | Timeout
  | DecodingError(string)

let showError = (err: error): string => {
  switch err {
  | Empty => "Expected JSON value but response was empty"
  | NetworkError => "Network error occured"
  | Timeout => "Request timeout"
  | DecodingError(err) => "Error decoding data " ++ err
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

let mapDecodingError = (res: result<'a, string>): result<'a, error> => {
  switch res {
  | Error(str) => Error(DecodingError(str))
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
  })
}

let fromRequest = (future, decoder) => {
  future
  ->Future.mapError(~propagateCancel=true, fromRequestError)
  ->Future.mapResult(~propagateCancel=true, emptyToError)
  ->Future.mapResult(~propagateCancel=true, res => res->Json.decode(decoder)->mapDecodingError)
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
