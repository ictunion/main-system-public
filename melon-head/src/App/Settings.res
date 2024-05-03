@module external styles: {..} = "./Settings/styles.module.scss"

open Belt

module ViewBool = {
  @react.component
  let make = (~value: bool) => {value ? React.string("👍 yes") : React.string("👎 no")}
}

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
let make = (~api: Api.t, ~session: Api.webData<Session.t>) => {
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
      "View member detail",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ViewMember)} />},
    ),
    (
      "Manage members",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
    (
      "Manage realm users",
      session => {<ViewBool value={Session.hasRealmRole(session, ~role=Session.ManageUsers)} />},
    ),
    (
      "Super-Powers (be careful!)",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
  ]

  <Page>
    <Page.Title> {React.string("Settings")} </Page.Title>
    <div className={styles["grid"]}>
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
