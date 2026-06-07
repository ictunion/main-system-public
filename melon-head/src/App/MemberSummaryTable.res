open Data
open Belt

let viewPaddedNumber = (n: int): React.element => {
  let stringified = Int.toString(n)
  let prefix = "0000000"
  let shortenedPrefix = prefix->Js.String.slice(~from=0, ~to_=7 - String.length(stringified))
  let style = ReactDOM.Style.make(~color="rgba(var(--rgb_color1), 0.7)", ())

  <span>
    <span style> {React.string(shortenedPrefix)} </span>
    {React.string(stringified)}
  </span>
}

let viewNote = (r: MemberData.summary, onNoteClick: (Uuid.t, option<string>) => unit) => {
  let text = switch r.note {
  | Some(n) if n !== "" => n
  | _ => "Add note"
  }
  <a onClick={_ => onNoteClick(r.id, r.note)}> {React.string(text)} </a>
}

@react.component
let make = (
  ~data: Api.webData<array<MemberData.summary>>,
  ~onNoteClick: option<(Uuid.t, option<string>) => unit>=?,
  ~showLeftOn: bool=false,
  ~children=React.null,
) => {
  let noteColumn = onNoteClick->Option.map((handler): DataTable.column<MemberData.summary> => {
    name: "Note",
    minMax: ("250px", "10fr"),
    view: r => viewNote(r, handler),
  })

  let leftOnColumn: option<DataTable.column<MemberData.summary>> = if showLeftOn {
    Some({
      name: "Left On",
      minMax: ("150px", "1fr"),
      view: r => r.leftAt->View.option(d => React.string(d->Js.Date.toLocaleDateString)),
    })
  } else {
    None
  }

  let baseStart: array<DataTable.column<MemberData.summary>> = [
    {
      name: "ID",
      minMax: ("100px", "1fr"),
      view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
    },
    {
      name: "Member Number",
      minMax: ("200px", "1fr"),
      view: r => viewPaddedNumber(r.memberNumber),
    },
    {
      name: "First Name",
      minMax: ("150px", "2fr"),
      view: r => r.firstName->View.option(React.string),
    },
    {
      name: "Last Name",
      minMax: ("150px", "2fr"),
      view: r => r.lastName->View.option(React.string),
    },
  ]

  let baseEnd: array<DataTable.column<MemberData.summary>> = [
    {
      name: "Last Company",
      minMax: ("220px", "2fr"),
      view: r => r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
    },
    {
      name: "City",
      minMax: ("250px", "1fr"),
      view: r => r.city->View.option(React.string),
    },
    {
      name: "Created On",
      minMax: ("150px", "1fr"),
      view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
    },
  ]

  let columns = Array.concatMany([
    baseStart,
    noteColumn->Option.mapWithDefault([], col => [col]),
    baseEnd,
    leftOnColumn->Option.mapWithDefault([], col => [col]),
  ])

  <DataTable data columns> children </DataTable>
}
