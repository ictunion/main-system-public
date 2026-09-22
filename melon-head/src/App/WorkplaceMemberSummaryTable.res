open Data
open Belt

/* Reduced member table for workplace executive committees, backed by
   WorkplaceData.mineMember (the /workplaces/mine/members projection). Shared
   by MyWorkplace (the roster) and MyWorkplaceMissingDues (the same roster
   narrowed to members behind on dues) so both render identically. */
@react.component
let make = (
  ~data: Api.webData<array<WorkplaceData.mineMember>>,
  ~extraColumns: array<DataTable.column<WorkplaceData.mineMember>>=[],
  ~children=React.null,
) => {
  let baseColumns: array<DataTable.column<WorkplaceData.mineMember>> = [
    {
      name: "ID",
      minMax: ("100px", "1fr"),
      view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
    },
    {
      name: "Member Number",
      minMax: ("200px", "1fr"),
      view: r =>
        MemberSummaryTable.viewPaddedNumber(
          r.memberNumber,
          ~isRepresentative=Some(r.isRepresentative),
          (),
        ),
    },
    {
      name: "First Name",
      minMax: ("150px", "1fr"),
      view: r => r.firstName->View.option(React.string),
    },
    {
      name: "Last Name",
      minMax: ("150px", "1fr"),
      view: r => r.lastName->View.option(React.string),
    },
    {
      name: "Email",
      minMax: ("250px", "3fr"),
      view: r => r.email->View.option(e => React.string(Email.toString(e))),
    },
    {
      name: "Phone",
      minMax: ("180px", "2fr"),
      view: r => r.phoneNumber->View.option(p => React.string(PhoneNumber.toString(p))),
    },
  ]

  let createdAtColumn: array<DataTable.column<WorkplaceData.mineMember>> = [
    {
      name: "Created On",
      minMax: ("150px", "1fr"),
      view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
    },
  ]

  let columns = Array.concatMany([baseColumns, extraColumns, createdAtColumn])

  <DataTable data columns> children </DataTable>
}
