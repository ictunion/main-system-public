@module external styles: {..} = "./Members/styles.module.scss"

open Data

@react.component
let make = (~api: Api.t, ~id, ~modal) => {
  let (currentWorkplace, _, _) =
    api->Hook.getData(
      ~path="/workplaces/" ++ Uuid.toString(id),
      ~decoder=WorkplaceData.Decode.summary,
    )

  let (workplaceMembers, _, refreshWorkplaceMembers) =
    api->Hook.getData(
      ~path="/workplaces/" ++ Uuid.toString(id) ++ "/members",
      ~decoder=Json.Decode.array(MemberData.Decode.summary),
    )

  let openNewNoteModal = (uuid, note) =>
    Modal.Interface.openModal(
      modal,
      NewNote.Member.modal(~api, ~modal, ~refresh=refreshWorkplaceMembers, uuid, note),
    )

  let onExportCsv = (members: array<MemberData.summary>, _) => {
    let workplaceName = currentWorkplace->RemoteData.unwrap(~default="workplace", s => s.name)
    let safeName =
      workplaceName
      ->String.split("")
      ->Array.map(c =>
        if RegExp.test(/[a-zA-Z0-9\-]/, c) {
          c
        } else {
          "_"
        }
      )
      ->Array.joinUnsafe("")
    let rows = members->Array.map(m => {
      let email = m.email->Option.mapOr("", Data.Email.toString)
      let name = Option.getOr(m.firstName, "") ++ " " ++ Option.getOr(m.lastName, "")
      let attributes =
        "{\"lang\": \"" ++
        Option.getOr(m.language, "") ++
        "\", \"memberNumber\": \"" ++
        Int.toString(m.memberNumber) ++ "\"}"
      [email, name, attributes]
    })
    let csv = Papa.unparse(Array.concat([["email", "name", "attributes"]], rows))
    let date = Date.make()->Data.Day.toString
    Download.csv(~filename="workplace-" ++ safeName ++ "-members-" ++ date ++ ".csv", ~content=csv)
  }

  <Page requireAnyRole=[ListWorkplaces]>
    <Page.Title>
      {React.string(
        "Members from " ++
        currentWorkplace->RemoteData.unwrap(~default="cannot happen", s => s.name),
      )}
    </Page.Title>
    <SessionContext.RequireRole anyOf=[Session.ListWorkplaces]>
      <Page.BackButton name="workplaces" path="/workplaces" />
    </SessionContext.RequireRole>
    <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
      {switch workplaceMembers->RemoteData.toOption {
      | None => React.null
      | Some(members) if Array.length(members) === 0 => React.null
      | Some(members) =>
        <Button.Panel>
          <Button onClick={onExportCsv(members, ...)}>
            {React.string("Export Listmonk CSV")}
          </Button>
        </Button.Panel>
      }}
    </SessionContext.RequireRole>
    <MemberSummaryTable
      data=workplaceMembers
      onNoteClick={(id, note) => openNewNoteModal(id, note)}
      columns=MemberSummaryTable.defaultColumns
    >
      <p> {React.string("There are no members yet.")} </p>
    </MemberSummaryTable>
  </Page>
}
