@module external styles: {..} = "./Dashboard/styles.module.scss"

open Belt

let applicationsRows: array<RowBasedTable.row<StatsData.Applications.basic>> = [
  ("Processing", s => React.string(s.processing->Int.toString)),
  ("Pending Verification", s => React.string(s.unverified->Int.toString)),
  ("Accepted", s => React.string(s.accepted->Int.toString)),
  ("Rejected", s => React.string(s.rejected->Int.toString)),
  ("Invalid", s => React.string(s.invalid->Int.toString)),
  ("All", s => React.string(StatsData.Applications.all(s)->Int.toString)),
]

let membersRows: array<RowBasedTable.row<StatsData.Members.basic>> = [
  ("New", s => React.string(s.new->Int.toString)),
  ("Current", s => React.string(s.current->Int.toString)),
  ("Past", s => React.string(s.past->Int.toString)),
  ("All", s => React.string(StatsData.Members.all(s)->Int.toString)),
]

@react.component
let make = (
  ~session: Api.webData<Session.t>,
  ~setSessionState,
  ~api: Api.t,
  ~modal: Modal.Interface.t,
) => {
  let (applicationsBasicStats, _, _) =
    api->Hook.getData(
      ~path="/stats/applications/basic",
      ~decoder=StatsData.Applications.Decode.basic,
    )

  let (membersBasicStats, _, _) =
    api->Hook.getData(~path="/stats/members/basic", ~decoder=StatsData.Members.Decode.basic)

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
        <RowBasedTable
          rows=applicationsRows data=applicationsBasicStats title=Some("Applications Stats")
        />
        <a onClick={openLink("/applications")} className={styles["pageLink"]}>
          {React.string("See Applications")}
        </a>
      </div>
      <div className={styles["gridItem"]}>
        <h2 className={styles["itemTitle"]}> {React.string("Members")} </h2>
        <RowBasedTable rows=membersRows data=membersBasicStats title=Some("Members Stats") />
        <a onClick={openLink("/members")} className={styles["pageLink"]}>
          {React.string("See Members")}
        </a>
      </div>
    </div>
  </Page>
}
