@module external styles: {..} = "./App/styles.module.scss"

module ConfiguredApp = {
  @react.component
  let make = (~keycloak: Keycloak.t, ~config: Config.t) => {
    let api = Api.make(~config, ~keycloak)

    let (sessionState: Api.webData<Session.t>, setSessionState) = React.useState(RemoteData.init)

    let (isNavOpen, setIsNavOpen) = React.useState(_ => false)
    let (isProfileOpen, setIsProfileOpen) = React.useState(_ => false)

    let url = RescriptReactRouter.useUrl()

    React.useEffect1(() => {
      let req = api->Api.getJson(~path="/session/current", ~decoder=Session.Decode.session)
      setSessionState(RemoteData.setLoading)

      req->Future.get(res => {
        setSessionState(_ => RemoteData.fromResult(res))
      })
      Some(() => Future.cancel(req))
    }, [keycloak])

    <div className={styles["root"]}>
      {if isNavOpen {
        <nav className={styles["app-nav"]} />
      } else {
        React.null
      }}
      <div className={styles["main-container"]}>
        <AppHeader
          toggleNav={_ => setIsNavOpen(v => !v)}
          isNavOpen={isNavOpen}
          openProfile={_ => setIsProfileOpen(_ => true)}
        />
        /* Routing to pages */
        {switch url.path {
        | list{} => <Dashboard session=sessionState api />
        | _ => <PageNotFound />
        }}
      </div>
      {if isProfileOpen {
        <Profile
          closeProfile={_ => setIsProfileOpen(_ => false)} session=sessionState config keycloak
        />
      } else {
        React.null
      }}
    </div>
  }
}

module App = {
  @react.component
  let make = (~keycloak: Keycloak.t, ~config: Js.Json.t) => {
    switch Config.make(config) {
    | Ok(config) => <ConfiguredApp keycloak={keycloak} config={config} />
    | Error(err) =>
      <div>
        <h1> {React.string("Error Loading Application Configuration")} </h1>
        <pre> {React.string(err)} </pre>
      </div>
    }
  }
}
