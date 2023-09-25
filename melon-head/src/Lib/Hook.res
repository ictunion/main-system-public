let getData = (api: Api.t, ~path: string, ~decoder: Json.Decode.t<'a>) => {
  let (data: Api.webData<'a>, setData) = React.useState(RemoteData.init)

  React.useEffect0(() => {
    let req = api->Api.getJson(~path, ~decoder)
    setData(RemoteData.setLoading)

    req->Future.get(res => {
      setData(_ => RemoteData.fromResult(res))
    })

    Some(() => Future.cancel(req))
  })

  (data, setData)
}
