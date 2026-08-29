open Data
open Belt

/* Roster of the caller's own workplace(s), for workplace executive committees.

   Both requests are unparameterised on purpose: the scope is resolved server
   side from the caller's Keycloak executive group membership, so there is no
   workplace ID in the URL for anyone to swap. The page cannot be pointed at a
   workplace the caller is not on the committee of, whatever the client does. */
@react.component
let make = (~api: Api.t) => {
  let (workplaces, _, _) =
    api->Hook.getData(
      ~path="/workplaces/mine",
      ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.workplace),
    )

  let (members, _, _) =
    api->Hook.getData(
      ~path="/workplaces/mine/members",
      ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.member),
    )

  /* An executive of several workplaces gets one combined roster, so the column
     is only worth showing when it actually distinguishes rows. */
  let workplaceNames =
    workplaces
    ->RemoteData.unwrap(~default=[], ws => ws)
    ->Array.reduce(Belt.Map.String.empty, (acc, w) =>
      acc->Belt.Map.String.set(Uuid.toString(w.id), w.name)
    )

  let showWorkplaceColumn = Belt.Map.String.size(workplaceNames) > 1

  let title = switch workplaces {
  | RemoteData.Success([]) => "My Workplace"
  | RemoteData.Success(ws) =>
    "Members of " ++ ws->Array.map(w => w.name)->Js.Array2.joinWith(", ")
  | _ => "My Workplace"
  }

  let workplaceColumn: array<DataTable.column<WorkplaceData.mineMember>> = if showWorkplaceColumn {
    [
      {
        name: "Workplace",
        minMax: ("200px", "2fr"),
        view: r =>
          workplaceNames
          ->Belt.Map.String.get(Uuid.toString(r.workplaceId))
          ->View.option(React.string),
      },
    ]
  } else {
    []
  }

  /* Names link to the regular member detail page. Orca authorises that route per
     member: an executive gets a redacted record for people in their own
     workplace and a 403 for anyone else, so following a link -- or hand-editing
     the UUID in it -- cannot reach outside the committee's scope. */
  let toMember = (id: Uuid.t) => "/members/" ++ Uuid.toString(id)

  let viewName = (r: WorkplaceData.mineMember, name: option<string>) =>
    switch name {
    | Some(name) => {
        let path = toMember(r.id)
        let openPath = (e: JsxEvent.Mouse.t) => {
          e->JsxEvent.Mouse.preventDefault
          RescriptReactRouter.push(path)
        }
        <a href={path} onClick={openPath}> {React.string(name)} </a>
      }
    | None => React.null
    }

  let baseColumns: array<DataTable.column<WorkplaceData.mineMember>> = [
    /* Keeps a way through to the detail page for members who have no name on
       record, where both name cells would otherwise be blank. */
    {
      name: "Id",
      minMax: ("100px", "1fr"),
      view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
    },
    {
      name: "First Name",
      minMax: ("150px", "2fr"),
      view: r => viewName(r, r.firstName),
    },
    {
      name: "Last Name",
      minMax: ("150px", "2fr"),
      view: r => viewName(r, r.lastName),
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
      name: "Member Since",
      minMax: ("150px", "1fr"),
      view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
    },
  ]

  let columns = Array.concatMany([baseColumns, workplaceColumn, createdAtColumn])

  <Page requireAnyRole=[ListOwnWorkplaceMembers]>
    <Page.Title> {React.string(title)} </Page.Title>
    {switch workplaces {
    | RemoteData.Success([]) =>
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
