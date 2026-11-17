@module external styles: {..} = "./DataTable/styles.module.scss"

/* Makes a column's header a sort switch. The table doesn't sort anything
 itself -- the caller orders the rows and says which column is active. */
type sort = {
  active: bool,
  onSelect: unit => unit,
}

type column<'t> = {
  name: string,
  minMax: (string, string),
  view: 't => React.element,
  sort?: sort,
}

module HeadCell = {
  @react.component
  let make = (~column: column<'t>) => {
    <div className={styles["headCell"]}>
      {switch column.sort {
      | None => React.string(column.name)
      | Some({active, onSelect}) =>
        <button
          className={styles["sortBtn"] ++ (active ? " " ++ styles["sortBtnActive"] : "")}
          ariaPressed={active ? #"true" : #"false"}
          onClick={_ => onSelect()}
        >
          {React.string(column.name)}
        </button>
      }}
    </div>
  }
}

module THead = {
  @react.component
  let make = (~columns: array<column<'t>>) => {
    <div className={styles["thead"]}>
      {React.array(
        Array.mapWithIndex(columns, (column, index) =>
          <HeadCell key={Int.toString(index)} column />
        ),
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
        Array.mapWithIndex(columns, (column, index) => <Td key={Int.toString(index)} column row />),
      )}
    </div>
  }
}

/* `rowKey` gives each row a stable identity. Without it rows are keyed by
   position, so after a re-sort a row's state (e.g. an already-fetched cell
   preview) stays put and ends up showing on a different record. */
@react.component
let make = (
  ~data: Api.webData<array<'t>>,
  ~columns: array<column<'t>>,
  ~rowKey: option<'t => string>=?,
  ~children=React.null,
) => {
  let gridTemplate: string = Array.map(columns, r => {
    let (min, max) = r.minMax
    "minmax(" ++ min ++ "," ++ max ++ ") "
  })->Array.join("")

  let style = ReactDOM.Style.unsafeAddProp({}, "--grid-template-columns", gridTemplate)

  <div className={styles["root"]} style>
    <div className={styles["scrollContainer"]}>
      <THead columns />
      {switch data {
      | Success(rows) =>
        if Array.length(rows) == 0 {
          <div className={styles["emptyContainer"]}> children </div>
        } else {
          React.array(
            rows->Array.mapWithIndex((row, index) => {
              let key = rowKey->Option.mapOr(Int.toString(index), f => f(row))
              <Tr key columns row />
            }),
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
