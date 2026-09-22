open Data
open Belt

let viewPaddedNumber = (n: int, ~isRepresentative: option<bool>=None, ()): React.element => {
  let stringified = Int.toString(n)
  let prefix = "0000000"
  let shortenedPrefix = prefix->Js.String.slice(~from=0, ~to_=7 - String.length(stringified))
  let style = ReactDOM.Style.make(~color="rgba(var(--rgb_color1), 0.7)", ())

  <span>
    <span style> {React.string(shortenedPrefix)} </span>
    {React.string(stringified)}
    {switch isRepresentative {
    | Some(true) =>
      <span title="Workplace representative" style={ReactDOM.Style.make(~color="#E8B322", ())}>
        {React.string(" ★")}
      </span>
    | _ => React.null
    }}
  </span>
}

let viewNote = (r: MemberData.summary, onNoteClick: (Uuid.t, option<string>) => unit) => {
  let text = switch r.note {
  | Some(n) if n !== "" => n
  | _ => "Add note"
  }
  <a onClick={_ => onNoteClick(r.id, r.note)}> {React.string(text)} </a>
}

/* One entry per column a caller may want to show, in the order they want it
   shown. `Custom` is the escape hatch for a page-specific column (e.g. the
   missed-dues count on the Missing Dues page) that doesn't belong in this
   shared list. */
type column =
  | Id
  | MemberNumber
  | FirstName
  | LastName
  | Note
  | LastCompany
  | City
  | Email
  | Phone
  | CreatedOn
  | LeftOn
  | Custom(DataTable.column<MemberData.summary>)

/* What most member listings show, in order. Callers with page-specific needs
   (extra columns, a different set) build their own array instead. */
let defaultColumns: array<column> = [
  Id,
  MemberNumber,
  FirstName,
  LastName,
  Note,
  LastCompany,
  City,
  CreatedOn,
]

@react.component
let make = (
  ~data: Api.webData<array<MemberData.summary>>,
  ~columns: array<column>,
  ~onNoteClick: option<(Uuid.t, option<string>) => unit>=?,
  ~children=React.null,
) => {
  let toColumn = (c: column): option<DataTable.column<MemberData.summary>> =>
    switch c {
    | Id =>
      Some({
        name: "ID",
        minMax: ("100px", "1fr"),
        view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
      })
    | MemberNumber =>
      Some({
        name: "Member Number",
        minMax: ("200px", "1fr"),
        view: r => viewPaddedNumber(r.memberNumber, ~isRepresentative=r.isRepresentative, ()),
      })
    | FirstName =>
      Some({
        name: "First Name",
        minMax: ("150px", "2fr"),
        view: r => r.firstName->View.option(React.string),
      })
    | LastName =>
      Some({
        name: "Last Name",
        minMax: ("150px", "2fr"),
        view: r => r.lastName->View.option(React.string),
      })
    | Note =>
      onNoteClick->Option.map((handler): DataTable.column<MemberData.summary> => {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r => viewNote(r, handler),
      })
    | LastCompany =>
      Some({
        name: "Last Company",
        minMax: ("220px", "2fr"),
        view: r => r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
      })
    | City =>
      Some({
        name: "City",
        minMax: ("250px", "1fr"),
        view: r => r.city->View.option(React.string),
      })
    | Email =>
      Some({
        name: "Email",
        minMax: ("250px", "3fr"),
        view: r => r.email->View.option(e => React.string(Data.Email.toString(e))),
      })
    | Phone =>
      Some({
        name: "Phone",
        minMax: ("180px", "2fr"),
        view: r => r.phoneNumber->View.option(p => React.string(Data.PhoneNumber.toString(p))),
      })
    | CreatedOn =>
      Some({
        name: "Created On",
        minMax: ("150px", "1fr"),
        view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
      })
    | LeftOn =>
      Some({
        name: "Left On",
        minMax: ("150px", "1fr"),
        view: r => r.leftAt->View.option(d => React.string(d->Js.Date.toLocaleDateString)),
      })
    | Custom(col) => Some(col)
    }

  let resolvedColumns = columns->Array.keepMap(toColumn)

  <DataTable data columns=resolvedColumns> children </DataTable>
}
