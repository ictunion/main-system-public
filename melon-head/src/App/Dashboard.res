@module external styles: {..} = "./Dashboard/styles.module.scss"

open Belt

module ViewBool = {
  @react.component
  let make = (~value: bool) => {value ? React.string("👍 yes") : React.string("👎 no")}
}

let applicationsRows: array<RowBasedTable.row<StatsData.basic>> = [
  ("Processing", s => React.string(s.processing->Int.toString)),
  ("Pending Verification", s => React.string(s.unverified->Int.toString)),
  ("Accepted", s => React.string(s.accepted->Int.toString)),
  ("Rejected", s => React.string(s.rejected->Int.toString)),
  ("All", s => React.string(StatsData.all(s)->Int.toString)),
]

let statusRows: array<RowBasedTable.row<Api.status>> = [
  ("Http Status", s => React.string(s.httpStatus->Int.toString)),
  ("Http Message", s => React.string(s.httpMessage)),
  (
    "Keycloak Connected",
    ({authorizationConnected}) => {<ViewBool value={authorizationConnected} />},
  ),
  ("Database Connected", ({databaseConnected}) => {<ViewBool value={databaseConnected} />}),
]

@react.component
let make = (~session: Api.webData<Session.t>, ~setSessionState, ~api: Api.t) => {
  let (basicStats, _) = api->Hook.getData(~path="/stats/basic", ~decoder=StatsData.Decode.basic)
  let (status, _) = api->Hook.getData(~path="/status", ~decoder=Api.Decode.status)

  let permissionsRows = [
    (
      "Recognized as member",
      (session: Session.t) => {<ViewBool value={session.memberId->Option.isSome} />},
    ),
    (
      "List all applications",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListApplications)} />},
    ),
    (
      "View application detail",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ViewApplication)} />},
    ),
    (
      "Resolve (Approve or Reject) applications",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ResolveApplications)} />},
    ),
    (
      "List all members",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListMembers)} />},
    ),
  ]

  let openLink = (path: string, _) => {
    RescriptReactRouter.push(path)
  }

  let (error, setError) = React.useState(() => None)

  let doPair = _ => {
    //TODO: implement me

    let req =
      api->Api.postJson(
        ~path="/session/current/pair-by-email",
        ~decoder=Session.Decode.session,
        ~body=Js.Json.null,
      )

    req->Future.get(res => {
      switch res {
      | Ok(data) => setSessionState(_ => RemoteData.Success(data))
      | Error(err) => setError(_ => Some(err))
      }
    })
  }

  <Page>
    <Page.Title> {React.string("Dashboard")} </Page.Title>
    <div className={styles["welcome"]}>
      <h3>
        {React.string("Hello ")}
        {React.string(session->RemoteData.unwrap(~default="--", s => s.tokenClaims.name))}
        {React.string("!")}
      </h3>
      {switch session->RemoteData.map(s => s.memberId) {
      | Success(None) =>
        <Message.Warning>
          <Message.Title>
            {React.string("Your account is not paired with any member!")}
          </Message.Title>
          <p>
            {React.string(
              "Some functions won't be accessible unless you pair your account. Just be aware that you might need to create member if it doesn't exist yet.",
            )}
          </p>
          <p>
            {React.string(
              "If you're not member but administrator you can safely ignore this message. Fuctions which require membership won't be available to you.",
            )}
          </p>
          <Message.ButtonPanel>
            <Button btnType=Button.Cta onClick=doPair>
              {React.string(
                "Pair account by email " ++
                session->RemoteData.unwrap(~default="canot happen", s =>
                  Data.Email.toString(s.tokenClaims.email)
                ),
              )}
            </Button>
          </Message.ButtonPanel>
        </Message.Warning>
      | Success(Some(uuid)) =>
        <p>
          {React.string("Your account is paired with member id " ++ Data.Uuid.toString(uuid))}
        </p>
      | _ => React.null
      }}
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
    </div>
    <div className={styles["statsGrid"]}>
      <div className={styles["gridItem"]}>
        <h2 className={styles["itemTitle"]}> {React.string("Applications")} </h2>
        <RowBasedTable rows=applicationsRows data=basicStats title=Some("Applications Stats") />
        <a onClick={openLink("/applications")} className={styles["pageLink"]}>
          {React.string("See Applications")}
        </a>
      </div>
      <div className={styles["gridItem"]}>
        <h2 className={styles["itemTitle"]}> {React.string("Permissions")} </h2>
        <RowBasedTable rows=permissionsRows data=session title=Some("Your Permissions/Roles") />
      </div>
      <div className={styles["gridItem"]}>
        <h2 className={styles["itemTitle"]}> {React.string("System")} </h2>
        <RowBasedTable rows=statusRows data=status title=Some("Api Status") />
      </div>
    </div>
  </Page>
}
