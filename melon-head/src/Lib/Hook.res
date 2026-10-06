/* Fetches on mount, and again when `enabled` flips to true -- for requests
   that depend on a permission or on data that isn't loaded yet. A change of
   `path` alone does not refetch. `api` is last so the optional `enabled` can be
   erased when omitted; callers pipe it in (`api->Hook.getData(~path, ~decoder)`). */
let getData = (~enabled=true, ~path: string, ~decoder: Json.Decode.t<'a>, api: Api.t) => {
  let (data: Api.webData<'a>, setData) = React.useState(RemoteData.init)

  let send = () => {
    let req = api->Api.getJson(~path, ~decoder)
    setData(RemoteData.setLoading)

    req->Future.get(res => {
      setData(_ => RemoteData.fromResult(res))
    })

    req
  }

  React.useEffect1(() => {
    if enabled {
      let req = send()

      Some(() => Future.cancel(req))
    } else {
      None
    }
  }, [enabled])

  (data, setData, send)
}
