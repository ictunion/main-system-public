type applicationsTab =
  | Processing
  | Unverified
  | Accepted
  | Rejected

@react.component
let make = (~api: Api.t) => {
  let tabHandlers = Tabbed.make(Processing)

  let (basicStats, setBasicStats) = React.useState(RemoteData.init)

  React.useEffect0(() => {
    let req = api->Api.getJson(~path="/stats/basic", ~decoder=Stats.Decode.basic)
    setBasicStats(RemoteData.setLoading)

    req->Future.get(res => {
      setBasicStats(_ => RemoteData.fromResult(res))
    })

    Some(() => Future.cancel(req))
  })

  <Page requireAnyRole=[ListApplications]>
    <Page.Title> {React.string("Applications")} </Page.Title>
    <div>
      <Tabbed.Tabs>
        <Tabbed.Tab value={Processing} handlers={tabHandlers}>
          <span> {React.string("Processing")} </span>
          <Chip.Count value={basicStats->RemoteData.map(r => r.processing)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Unverified} handlers={tabHandlers} color=Some("#a63ded")>
          {React.string("Pending Verification")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.unverified)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Accepted} handlers={tabHandlers} color=Some("#00c49f")>
          {React.string("Accepted")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.accepted)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Rejected} handlers={tabHandlers} color=Some("#ef562e")>
          {React.string("Rejected")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.rejected)} />
        </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab={Processing} handlers={tabHandlers}>
        {React.string("there will be content for 1st tab")}
      </Tabbed.Content>
      <Tabbed.Content tab={Unverified} handlers={tabHandlers}>
        {React.string("Other there will be content for 1st tab")}
      </Tabbed.Content>
    </div>
  </Page>
}
