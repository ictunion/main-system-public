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
  ("Proxy Support Enabled", ({proxySupportEnabled}) => {<ViewBool value={proxySupportEnabled} />}),
]

@react.component
let make = (
  ~session: Api.webData<Session.t>,
  ~setSessionState,
  ~api: Api.t,
  ~modal: Modal.Interface.t,
) => {
  let (basicStats, _, _) =
    api->Hook.getData(~path="/stats/applications/basic", ~decoder=StatsData.Decode.basic)
  let (status, _, _) = api->Hook.getData(~path="/status", ~decoder=Api.Decode.status)

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
    (
      "Manage members",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
    (
      "Super-Powers (be careful!)",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
  ]

  let openLink = (path: string, _) => {
    RescriptReactRouter.push(path)
  }

  let (error, setError) = React.useState(() => None)

  let doPair = _ => {
    let req =
      api->Api.postJson(
        ~path="/session/current/pair-by-email",
        ~decoder=Session.Decode.session,
        ~body=Js.Json.null,
      )

    req->Future.get(res => {
      switch res {
      | Ok(data) => {
          setSessionState(_ => RemoteData.Success(data))
          setError(_ => None)
        }
      | Error(err) => setError(_ => Some(err))
      }
    })
  }

  let openMember = uuid => {
    RescriptReactRouter.push("/members/" ++ uuid->Data.Uuid.toString)
  }

  let createMember = _ => {
    // TODO: once we have stats for members they should be refreshed there
    modal->Modal.Interface.openModal(Members.newMemberModal(~api, ~modal, ~refreshMembers=_ => ()))
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
            <p> {React.string("Your account is not paired with any member!")} </p>
          </Message.Title>
          <p>
            {React.string(
              "Some functions won't be accessible unless you pair your account. Just be aware that you might need to ",
            )}
            <a onClick=createMember> {React.string("create a new member")} </a>
            {React.string(" if it doesn't exist yet.")}
          </p>
          <p>
            {React.string(
              "If you're not a member, but an administrator, you can safely ignore this message. Fuctions which require membership won't be available to you.",
            )}
          </p>
          <Message.ButtonPanel>
            <Button variant=Button.Cta onClick=doPair>
              {React.string(
                "Pair account by email " ++
                session->RemoteData.unwrap(~default="canot happen", s =>
                  Data.Email.toString(s.tokenClaims.email)
                ),
              )}
            </Button>
          </Message.ButtonPanel>
          {switch error {
          | None => React.null
          | Some(err) =>
            <Message.Error>
              <Message.Title>
                {React.string("This didin't work as you would wish for...")}
              </Message.Title>
              {React.string(err->Api.showError)}
            </Message.Error>
          }}
        </Message.Warning>
      | Success(Some(uuid)) =>
        <Message.Info>
          <p>
            {React.string("Your account is paired with member id ")}
            <a className={styles["memberLink"]} onClick={_ => openMember(uuid)}>
              {React.string(Data.Uuid.toString(uuid))}
            </a>
          </p>
        </Message.Info>
      | _ => React.null
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
