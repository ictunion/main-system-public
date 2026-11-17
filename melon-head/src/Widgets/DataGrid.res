@module external styles: {..} = "./DataGrid/styles.module.scss"

module Cell = {
  @react.component
  let make = (~label: option<string>=?, ~children: React.element) => {
    <div className={styles["cell"]}>
      {switch label {
      | None => React.null
      | Some(l) => <span className={styles["label"]}> {React.string(l)} </span>
      }}
      <div> {children} </div>
    </div>
  }
}

module Loading = {
  @react.component
  let make = () => {
    <div className={styles["loading"]} />
  }
}

type cell<'a> = {
  label: string,
  view: 'a => React.element,
  minmax: (string, string),
}

type row<'a> = {
  label: string,
  cells: array<cell<'a>>,
}

type t<'a> = array<row<'a>>

let viewRow = (row: row<'a>, rowIndex, ~data: Api.webData<'a>) => {
  let gridTemplate: string = Array.map(row.cells, r => {
    let (min, max) = r.minmax
    "minmax(" ++ min ++ "," ++ max ++ ") "
  })->Array.join("")

  let style = ReactDOM.Style.unsafeAddProp({}, "--grid-template-columns", gridTemplate)

  <div key={Int.toString(rowIndex)} className={styles["rowWrap"]}>
    {if row.label != "" {
      <h3 className={styles["rowLabel"]}> {React.string(row.label)} </h3>
    } else {
      React.null
    }}
    <div style key={Int.toString(rowIndex)} className={styles["row"]}>
      {row.cells
      ->Array.mapWithIndex((c, index) =>
        <Cell key={Int.toString(index)} label={c.label}>
          {switch data {
          | Success(d) => c.view(d)
          | Idle => <Loading />
          | Loading => <Loading />
          | Failure(_) => React.string("[error]")
          }}
        </Cell>
      )
      ->React.array}
    </div>
  </div>
}

@react.component
let make = (~layout: t<'a>, ~data: Api.webData<'a>) =>
  <div className={styles["dataGrid"]}>
    {layout->Array.mapWithIndex(viewRow(~data, ...))->React.array}
    {switch data {
    | Failure(err) => <div className={styles["error"]}> {React.string(err->Api.showError)} </div>
    | _ => React.null
    }}
  </div>
