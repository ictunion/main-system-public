@module external styles: {..} = "./Members/styles.module.scss"

module NewWorkplace = {
  open WorkplaceData

  let emptyWorkplace = {
    name: "",
    email: "",
  }

  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~refreshWorkplaces) => {
    let (newWorkplace, setNewWorkplace) = React.useState(_ => emptyWorkplace)
    let (createMore, setCreateMore) = React.useState(_ => false)
    let (error, setError) = React.useState(() => None)

    let onSubmit = _ => {
      let body: Js.Json.t = WorkplaceData.Encode.newWorkplace(newWorkplace)
      let req = api->Api.postJson(~path="/workplaces", ~decoder=WorkplaceData.Decode.summary, ~body)

      req->Future.get(res => {
        switch res {
        | Ok(_data) => {
            let _ = refreshWorkplaces()
            if createMore {
              setNewWorkplace(_ => emptyWorkplace)
            } else {
              Modal.Interface.closeModal(modal)
            }
          }
        | Error(e) => setError(_ => Some(e))
        }
      })
    }

    <Form onSubmit>
      <Form.TextField
        label="Email"
        placeholder="evilcorp@ictunion.cz"
        value=newWorkplace.email
        onInput={email => setNewWorkplace(m => {...m, email})}
      />
      <Form.TextField
        label="Name"
        placeholder="Evil corp."
        value=newWorkplace.name
        onInput={name => setNewWorkplace(m => {...m, name})}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Create New Workplace")} </Button>
        <Form.HorizontalLabel>
          <Form.Checkbox checked=createMore onChange={_ => setCreateMore(v => !v)} />
          {React.string("Create another workplace")}
        </Form.HorizontalLabel>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
    </Form>
  }
}

let newWorkplaceModal = (~api, ~modal, ~refreshWorkplaces): Modal.modalContent => {
  title: "Add New Workplace",
  content: <NewWorkplace api modal refreshWorkplaces />,
}

let columns: array<DataTable.column<WorkplaceData.summary>> = [
  {
    name: "ID",
    minMax: ("100px", "1fr"),
    view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
  },
  {
    name: "Name",
    minMax: ("150px", "2fr"),
    view: r => r.name->React.string,
  },
  {
    name: "Email",
    minMax: ("250px", "2fr"),
    view: r => r.email->(email => <Link.Email email />),
  },
  {
    name: "Created On",
    minMax: ("150px", "1fr"),
    view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
  },
]

// We could use Functor module to generate these but would probably make it harder for people to understand what is going on

module All = {
  @react.component
  let make = (~api, ~modal) => {
    let (workplaces, _, refreshWorkplaces) =
      api->Hook.getData(
        ~path="/workplaces",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )

    let openNewWorkplaceModal = _ =>
      Modal.Interface.openModal(modal, newWorkplaceModal(~api, ~modal, ~refreshWorkplaces))

    <React.Fragment>
      <Button.Panel>
        <Button onClick=openNewWorkplaceModal> {React.string("Add New Workplace")} </Button>
      </Button.Panel>
      <DataTable
        data=workplaces
        columns=[
          {
            name: "ID",
            minMax: ("100px", "1fr"),
            view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/workplaces/" ++ uuid} />,
          },
          {
            name: "Name",
            minMax: ("150px", "2fr"),
            view: r => r.name->React.string,
          },
          {
            name: "Email",
            minMax: ("250px", "2fr"),
            view: r => r.email->(email => <Link.Email email />),
          },
          {
            name: "Created On",
            minMax: ("150px", "1fr"),
            view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
          },
        ]>
        <p> {React.string("Currently there are no workplaces.")} </p>
      </DataTable>
    </React.Fragment>
  }
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Workplaces")} </Page.Title>
    <All api modal />
  </Page>
}
