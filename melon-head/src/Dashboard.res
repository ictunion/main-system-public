@module external styles: {..} = "./Dashboard/styles.module.scss"

open Belt

module ViewBool = {
  @react.component
  let make = (~value: bool) => {value ? React.string("👍 yes") : React.string("👎 no")}
}

module RowBasedTable = {
  @react.component
  let make = (~rows: array<(string, 'a => React.element)>, ~data: Api.webData<'a>, ~title=None) => {
    let viewRow = (index, (name, getter)) =>
      <tr key={Int.toString(index)}>
        <td> {React.string(name)} </td>
        <td> {data->RemoteData.unwrap(getter, ~default=React.null)} </td>
      </tr>

    let viewRows = rows->Array.mapWithIndex(viewRow)->React.array

    <div className={styles["row-table-wrapper"]}>
      {switch title {
      | Some(str) => <h2 className={styles["row-table-title"]}> {React.string(str)} </h2>
      | None => React.null
      }}
      <table className={styles["row-table-table"]}>
        <tbody> viewRows </tbody>
      </table>
      {switch data {
      | Loading =>
        <div className={styles["row-table-loading"]}>
          <Icons.Loading variant=Icons.Dark />
        </div>
      | Failure(err) =>
        <div className={styles["row-table-error"]}>
          <h4> {React.string("Error loading data")} </h4>
          <pre> {React.string(Api.showError(err))} </pre>
        </div>
      | _ => React.null
      }}
    </div>
  }
}

@react.component
let make = (~session: Api.webData<Session.t>, ~api: Api.t) => {
  open Stats

  let (basicStats, _) = api->Hook.getData(~path="/stats/basic", ~decoder=Stats.Decode.basic)
  let (status, _) = api->Hook.getData(~path="/status", ~decoder=Api.Decode.status)

  let applicationsRows = [
    ("Processing", ({processing}) => React.string(processing->Int.toString)),
    ("Pending Verification", ({unverified}) => React.string(unverified->Int.toString)),
    ("Accepted", ({accepted}) => React.string(accepted->Int.toString)),
    ("Rejected", ({rejected}) => React.string(rejected->Int.toString)),
  ]

  let permissionsRows = [
    (
      "List all applications in the system",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListApplications)} />},
    ),
  ]

  open Api
  let statusRows = [
    ("Http Status", ({httpStatus}) => React.string(httpStatus->Int.toString)),
    ("Http Message", ({httpMessage}) => React.string(httpMessage)),
    (
      "Keycloak Connceted",
      ({authorizationConnected}) => {<ViewBool value={authorizationConnected} />},
    ),
    ("Database Connceted", ({databaseConnected}) => {<ViewBool value={databaseConnected} />}),
  ]

  <Page>
    <Page.Title> {React.string("Dashboard")} </Page.Title>
    <div className={styles["stats-grid"]}>
      <RowBasedTable rows=applicationsRows data=basicStats title=Some("Current Applications") />
      <RowBasedTable rows=permissionsRows data=session title=Some("Your Permissions/Roles") />
      <RowBasedTable rows=statusRows data=status title=Some("Api Status") />
    </div>
  </Page>
}
