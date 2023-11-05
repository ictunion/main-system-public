@module external styles: {..} = "./View/styles.module.scss"

@scope("window") @val external window_open: (string, string) => unit = "open"

open Belt

let option = (o: option<'a>, view: 'a => React.element): React.element => {
  switch o {
  | Some(v) => view(v)
  | None => React.string("---")
  }
}

let fileRow = (~api: Api.t, file: Data.file): React.element => {
  open Data
  let fileName = file.name ++ "." ++ file.fileType

  let openFile = _ => {
    let fileUrl =
      api.host ++
      "/files/" ++
      file.id->Uuid.toString ++
      "?token=" ++
      api.keycloak->Keycloak.getToken

    window_open(fileUrl, "_blank")
  }

  <tr key={Uuid.toString(file.id)}>
    <td>
      <a onClick=openFile> {React.string(fileName)} </a>
    </td>
    <td> {file.createdAt->Js.Date.toLocaleString->React.string} </td>
  </tr>
}

let filesTable = (~api: Api.t, ~files: array<Data.file>): React.element => {
  <table className={styles["filesTable"]}>
    <thead>
      <tr>
        <td> {React.string("File Name")} </td>
        <td> {React.string("Created at")} </td>
      </tr>
    </thead>
    <tbody> {files->Array.map(fileRow(~api))->React.array} </tbody>
  </table>
}
