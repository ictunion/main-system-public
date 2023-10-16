open Belt

@module external styles: {..} = "./DataTable/styles.module.scss"

type column<'t> = {
  name: string,
  minMax: (string, string),
  view: 't => React.element,
}

module HeadCell = {
  @react.component
  let make = (~column: column<'t>) => {
    <div className={styles["headCell"]}> {React.string(column.name)} </div>
  }
}

module THead = {
  @react.component
  let make = (~columns: array<column<'t>>) => {
    <div className={styles["thead"]}>
      {React.array(
        Array.mapWithIndex(columns, (id, column) => <HeadCell key={Int.toString(id)} column />),
      )}
    </div>
  }
}

module Td = {
  @react.component
  let make = (~column: column<'t>, ~row: 't) => {
    let element = column.view(row)
    <div className={styles["td"]}> element </div>
  }
}

module Tr = {
  @react.component
  let make = (~columns: array<column<'t>>, ~row: 't) => {
    <div className={styles["tr"]}>
      {React.array(
        Array.mapWithIndex(columns, (id, column) => <Td key={Int.toString(id)} column row />),
      )}
    </div>
  }
}

@react.component
let make = (~data: Api.webData<array<'t>>, ~columns: array<column<'t>>, ~children=React.null) => {
  let gridTemplate: string = Array.map(columns, r => {
    let (min, max) = r.minMax
    "minmax(" ++ min ++ "," ++ max ++ ") "
  })->Js.String.concatMany("")

  let style =
    ReactDOM.Style.make()->ReactDOM.Style.unsafeAddProp("--grid-template-columns", gridTemplate)

  <div className={styles["root"]} style>
    <div className={styles["scrollContainer"]}>
      <THead columns />
      {switch data {
      | Success(rows) =>
        if Array.length(rows) == 0 {
          <div className={styles["emptyContainer"]}> children </div>
        } else {
          React.array(
            rows->Array.mapWithIndex((id, row) => <Tr key={Int.toString(id)} columns row />),
          )
        }
      | Failure(err) => <div className={styles["error"]}> {React.string(err->Api.showError)} </div>
      | Loading =>
        <div className={styles["loadingContainer"]}>
          <Icons.Loading variant=Icons.Dark />
        </div>
      | _ => React.null
      }}
    </div>
  </div>
}
