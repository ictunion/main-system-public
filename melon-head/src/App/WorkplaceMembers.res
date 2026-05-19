@module external styles: {..} = "./Members/styles.module.scss"

open Data
open Belt

let newNoteModal = (
  ~api,
  ~modal,
  ~refreshMembers,
  uuid,
  ~isApplication,
  initialNote,
): Modal.modalContent => {
  title: "Update note",
  content: <NewNote api modal refreshMembers uuid isApplication initialNote />,
}

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
      newNoteModal(
        ~api,
        ~modal,
        ~refreshMembers=refreshWorkplaceMembers,
        uuid,
        ~isApplication=false,
        note,
      ),
    )

  let onExportCsv = (members: array<MemberData.summary>, _) => {
    let workplaceName = currentWorkplace->RemoteData.unwrap(~default="workplace", s => s.name)
    let safeName =
      workplaceName
      ->Js.String2.split("")
      ->Belt.Array.map(c =>
        if Js.Re.test_(%re("/[a-zA-Z0-9\-]/"), c) {
          c
        } else {
          "_"
        }
      )
      ->Js.Array2.joinWith("")
    let rows = members->Belt.Array.map(m => {
      let email = m.email->Belt.Option.mapWithDefault("", Data.Email.toString)
      let name =
        Belt.Option.getWithDefault(m.firstName, "") ++
        " " ++
        Belt.Option.getWithDefault(m.lastName, "")
      let attributes =
        "{\"lang\": \"" ++
        Belt.Option.getWithDefault(m.language, "") ++
        "\", \"memberNumber\": \"" ++
        Belt.Int.toString(m.memberNumber) ++ "\"}"
      [email, name, attributes]
    })
    let csv = Papa.unparse(Belt.Array.concat([["email", "name", "attributes"]], rows))
    let date = Js.Date.make()->Js.Date.toJSONUnsafe->Js.String.slice(~from=0, ~to_=10)
    Download.csv(~filename="workplace-" ++ safeName ++ "-members-" ++ date ++ ".csv", ~content=csv)
  }

  <Page requireAnyRole=[ListWorkplaces]>
    <Page.Title>
      {React.string(
        "Members from " ++
        currentWorkplace->RemoteData.unwrap(~default="cannot happen", s => s.name),
      )}
    </Page.Title>
    <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
      {switch workplaceMembers->RemoteData.toOption {
      | None => React.null
      | Some(members) if Array.length(members) === 0 => React.null
      | Some(members) =>
        <Button.Panel>
          <Button onClick={onExportCsv(members)}> {React.string("Export Listmonk CSV")} </Button>
        </Button.Panel>
      }}
    </SessionContext.RequireRole>
    <MemberSummaryTable data=workplaceMembers onNoteClick={(id, note) => openNewNoteModal(id, note)}>
      <p> {React.string("There are no members yet.")} </p>
    </MemberSummaryTable>
  </Page>
}
