@module external styles: {..} = "./Members/styles.module.scss"

open Data
open Belt

let newNoteModal = (~api, ~modal, ~refreshMembers, uuid, ~isApplication, initialNote): Modal.modalContent => {
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
      newNoteModal(~api, ~modal, ~refreshMembers=refreshWorkplaceMembers, uuid, ~isApplication=false, note),
    )

  <Page requireAnyRole=[ListWorkplaces]>
    <Page.Title>
      {React.string(
        "Members from " ++
        currentWorkplace->RemoteData.unwrap(~default="cannot happen", s => s.name),
      )}
    </Page.Title>
    <DataTable
      data=workplaceMembers
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
        },
        {
          name: "Member Number",
          minMax: ("200px", "1fr"),
          view: r => Members.viewPaddedNumber(r.memberNumber),
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
        {
          name: "Note",
          minMax: ("250px", "10fr"),
          view: r =>
            <a onClick={_ => openNewNoteModal(r.id, r.note)}>
              {
                let note = if Option.getWithDefault(r.note, "Add note") == "" {
                  "Add note"
                } else {
                  Option.getWithDefault(r.note, "Add note")
                }

                React.string(note)
              }
            </a>,
        },
        {
          name: "Last Company",
          minMax: ("220px", "2fr"),
          view: r =>
            r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
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
      ]>
      <p> {React.string("There are no members yet.")} </p>
    </DataTable>
  </Page>
}
