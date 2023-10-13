let getData = (api: Api.t, ~path: string, ~decoder: Json.Decode.t<'a>) => {
  let (data: Api.webData<'a>, setData) = React.useState(RemoteData.init)

  let send = () => {
    let req = api->Api.getJson(~path, ~decoder)
    setData(RemoteData.setLoading)

    req->Future.get(res => {
      setData(_ => RemoteData.fromResult(res))
    })

    req
  }

  React.useEffect0(() => {
    let req = send()

    Some(() => Future.cancel(req))
  })

  (data, setData, send)
}
