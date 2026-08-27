@module external styles: {..} = "./Workplaces/styles.module.scss"

open Belt

type tab = Active | Inactive

module NewWorkplace = {
  open WorkplaceData

  let emptyWorkplace = {
    name: "",
    email: "",
    keycloakGroupId: "",
    keycloakExecutiveGroupId: "",
    newsletterId: "",
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
            refreshWorkplaces()
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
      <Form.TextField
        label="Keycloak ID"
        placeholder="acabcafe-ba11-4491-b29d-1aefccfddd93"
        value=newWorkplace.keycloakGroupId
        onInput={keycloakGroupId => setNewWorkplace(m => {...m, keycloakGroupId})}
      />
      <Form.TextField
        label="Executive Group Keycloak ID"
        placeholder="acabcafe-ba11-4491-b29d-1aefccfddd93"
        value=newWorkplace.keycloakExecutiveGroupId
        onInput={keycloakExecutiveGroupId => setNewWorkplace(m => {...m, keycloakExecutiveGroupId})}
      />
      <Form.TextField
        label="Newsletter ID (optional)"
        placeholder="42"
        value=newWorkplace.newsletterId
        onInput={newsletterId => setNewWorkplace(m => {...m, newsletterId})}
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
    view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/workplaces/" ++ uuid} />,
  },
  {
    name: "Name",
    minMax: ("150px", "2fr"),
    view: r => r.name->React.string,
  },
  {
    name: "",
    minMax: ("160px", "160px"),
    view: r =>
      <Button
        variant=Button.Cta
        onClick={_ =>
          RescriptReactRouter.push("/workplaces/" ++ Data.Uuid.toString(r.id) ++ "/members")}>
        {React.string("Members")}
      </Button>,
  },
  {
    name: "Email",
    minMax: ("250px", "2fr"),
    view: r => r.email->(email => <Link.Email email />),
  },
  {
    name: "Member count",
    minMax: ("50px", "1fr"),
    view: r => r.memberCount->(memberCount => React.string(memberCount->Int.toString)),
  },
  {
    name: "Status",
    minMax: ("120px", "1fr"),
    view: r =>
      switch WorkplaceData.getStatus(r) {
      | Initial => "Initial"
      | Established => "Established"
      | Announced => "Announced"
      | Cancelled => "Cancelled"
      }->React.string,
  },
]

module Active = {
  @react.component
  let make = (~api, ~refreshKey: int) => {
    let (workplaces, _, send) =
      api->Hook.getData(
        ~path="/workplaces/active",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )

    React.useEffect1(() => {
      if refreshKey > 0 {
        let _ = send()
      }
      None
    }, [refreshKey])

    <DataTable data=workplaces columns>
      <p> {React.string("There are no active workplaces.")} </p>
    </DataTable>
  }
}

module Inactive = {
  @react.component
  let make = (~api, ~refreshKey: int) => {
    let (workplaces, _, send) =
      api->Hook.getData(
        ~path="/workplaces/inactive",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )

    React.useEffect1(() => {
      if refreshKey > 0 {
        let _ = send()
      }
      None
    }, [refreshKey])

    <DataTable data=workplaces columns>
      <p> {React.string("There are no inactive workplaces.")} </p>
    </DataTable>
  }
}

module All = {
  @react.component
  let make = (~api, ~refreshKey: int) => {
    let (workplaces, _, send) =
      api->Hook.getData(
        ~path="/workplaces",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )

    React.useEffect1(() => {
      if refreshKey > 0 {
        let _ = send()
      }
      None
    }, [refreshKey])

    <DataTable data=workplaces columns>
      <p> {React.string("Currently there are no workplaces.")} </p>
    </DataTable>
  }
}

let urlToTab = (url: RescriptReactRouter.url): option<tab> => {
  switch url.hash {
  | "all" => None
  | "inactive" => Some(Inactive)
  | _ => Some(Active)
  }
}

let tabToUrl = (tab: option<tab>): string => {
  let hash = switch tab {
  | None => "all"
  | Some(Active) => "active"
  | Some(Inactive) => "inactive"
  }
  "/workplaces#" ++ hash
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  let (activeTab, setActiveTab_) = React.useState(_ =>
    RescriptReactRouter.dangerouslyGetInitialUrl()->urlToTab
  )

  let _ = RescriptReactRouter.watchUrl(url => {
    setActiveTab_(_ => urlToTab(url))
  })

  let setActiveTab = f => {
    let newTab = f(activeTab)
    RescriptReactRouter.push(tabToUrl(newTab))
  }

  let tabHandlers = (activeTab, setActiveTab)

  let (refreshKey, setRefreshKey) = React.useState(_ => 0)

  let refreshWorkplaces = () => setRefreshKey(k => k + 1)

  let openNewWorkplaceModal = _ =>
    Modal.Interface.openModal(modal, newWorkplaceModal(~api, ~modal, ~refreshWorkplaces))

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Workplaces")} </Page.Title>
    <Button.Panel>
      <Button onClick=openNewWorkplaceModal> {React.string("Add New Workplace")} </Button>
    </Button.Panel>
    <div className={styles["mainContent"]}>
      <Tabbed.Tabs>
        <Tabbed.Tab value={Some(Active)} handlers={tabHandlers}>
          {React.string("Active")}
        </Tabbed.Tab>
        <Tabbed.Tab value={Some(Inactive)} handlers={tabHandlers}>
          {React.string("Inactive")}
        </Tabbed.Tab>
        <Tabbed.TabSpacer />
        <Tabbed.Tab value={None} handlers={tabHandlers} color=Some("var(--color1)")>
          {React.string("All")}
        </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab={Some(Active)} handlers={tabHandlers}>
        <Active api refreshKey />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(Inactive)} handlers={tabHandlers}>
        <Inactive api refreshKey />
      </Tabbed.Content>
      <Tabbed.Content tab={None} handlers={tabHandlers}>
        <All api refreshKey />
      </Tabbed.Content>
    </div>
  </Page>
}
