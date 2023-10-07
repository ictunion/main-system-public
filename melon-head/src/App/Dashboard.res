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

  let openLink = (path: string, _) => {
    RescriptReactRouter.push(path)
  }

  <Page>
    <Page.Title> {React.string("Dashboard")} </Page.Title>
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
