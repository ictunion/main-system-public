@module external styles: {..} = "./RowBasedTable/styles.module.scss"

open Belt

type row<'a> = (string, 'a => React.element)

@react.component
let make = (~rows: array<row<'a>>, ~data: Api.webData<'a>, ~title=None) => {
  let viewRow = (index, (name, getter)) =>
    <tr key={Int.toString(index)}>
      <td> {React.string(name)} </td>
      <td> {data->RemoteData.unwrap(getter, ~default=React.null)} </td>
    </tr>

  let viewRows = rows->Array.mapWithIndex(viewRow)->React.array

  <div className={styles["root"]}>
    {switch title {
    | Some(str) => <h2 className={styles["rowTableTitle"]}> {React.string(str)} </h2>
    | None => React.null
    }}
    <table className={styles["rowTableTable"]}>
      <tbody> viewRows </tbody>
    </table>
    {switch data {
    | Loading =>
      <div className={styles["rowTableLoading"]}>
        <Icons.Loading variant=Icons.Dark />
      </div>
    | Failure(err) =>
      <div className={styles["rowTableRrror"]}>
        <h4> {React.string("Error loading data")} </h4>
        <pre> {React.string(Api.showError(err))} </pre>
      </div>
    | _ => React.null
    }}
  </div>
}
