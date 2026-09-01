open Data
open Belt

/* Roster of a workplace executive committee's own workplace.

   Both requests are unparameterised on purpose: the scope is resolved server
   side from the caller's Keycloak executive group membership, so there is no
   workplace ID in the URL for anyone to swap. */
@react.component
let make = (~api: Api.t) => {
  let (workplace, _, _) =
    api->Hook.getData(~path="/workplaces/mine", ~decoder=WorkplaceData.Decode.summary)

  let (members, _, _) =
    api->Hook.getData(
      ~path="/workplaces/mine/members",
      ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.member),
    )

  let title = switch workplace {
  | RemoteData.Success(w) => "Members of " ++ w.name
  | _ => "My Workplace"
  }

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

  let columns = Array.concatMany([baseColumns, createdAtColumn])

  <Page requireAnyRole=[ListOwnWorkplaceMembers]>
    <Page.Title> {React.string(title)} </Page.Title>
    {switch workplace {
    | RemoteData.Failure(_) =>
      <p>
        {React.string(
          "You are not on the executive committee of any workplace, so there is nothing to show here.",
        )}
      </p>
    | _ =>
      <DataTable data=members columns>
        <p> {React.string("There are no members in your workplace yet.")} </p>
      </DataTable>
    }}
  </Page>
}
