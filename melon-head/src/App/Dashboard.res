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
]

let statusRows: array<RowBasedTable.row<Api.status>> = [
  ("Http Status", s => React.string(s.httpStatus->Int.toString)),
  ("Http Message", s => React.string(s.httpMessage)),
  (
    "Keycloak Connceted",
    ({authorizationConnected}) => {<ViewBool value={authorizationConnected} />},
  ),
  ("Database Connceted", ({databaseConnected}) => {<ViewBool value={databaseConnected} />}),
]

@react.component
let make = (~session: Api.webData<Session.t>, ~api: Api.t) => {
  let (basicStats, _) = api->Hook.getData(~path="/stats/basic", ~decoder=StatsData.Decode.basic)
  let (status, _) = api->Hook.getData(~path="/status", ~decoder=Api.Decode.status)

  let permissionsRows = [
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
  ]

  <Page>
    <Page.Title> {React.string("Dashboard")} </Page.Title>
    <div className={styles["statsGrid"]}>
      <RowBasedTable rows=applicationsRows data=basicStats title=Some("Current Applications") />
      <RowBasedTable rows=permissionsRows data=session title=Some("Your Permissions/Roles") />
      <RowBasedTable rows=statusRows data=status title=Some("Api Status") />
    </div>
  </Page>
}
