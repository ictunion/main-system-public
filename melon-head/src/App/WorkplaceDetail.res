@module external styles: {..} = "./WorkplaceDetail/styles.module.scss"

open Data
open RemoteData

let layout: DataGrid.t<WorkplaceData.summary> = [
  {
    label: "Details",
    cells: [
      {
        label: "Name",
        view: d => d.name->React.string,
        minmax: ("200px", "900px"),
      },
      {
        label: "State",
        view: d =>
          switch WorkplaceData.getStatus(d) {
          | Initial => "Initial"
          | Established => "Established"
          | Announced => "Announced"
          | Cancelled => "Cancelled"
          }->React.string,
        minmax: ("120px", "200px"),
      },
      {
        label: "Email",
        view: d => <Link.Email email={d.email} />,
        minmax: ("300px", "900px"),
      },
      {
        label: "Members",
        view: d => d.memberCount->Belt.Int.toString->React.string,
        minmax: ("100px", "200px"),
      },
    ],
  },
  {
    label: "Keycloak",
    cells: [
      {
        label: "Group ID",
        view: d => d.keycloakGroupId->Uuid.toString->React.string,
        minmax: ("300px", "900px"),
      },
      {
        label: "Executive Group ID",
        view: d =>
          d.keycloakExecutiveGroupId->Belt.Option.mapWithDefault(React.string("—"), uuid =>
            uuid->Uuid.toString->React.string
          ),
        minmax: ("300px", "900px"),
      },
    ],
  },
  {
    label: "Newsletter",
    cells: [
      {
        label: "Listmonk ID",
        view: d =>
          switch d.newsletterId {
          | None => React.string("Newsletter not connected")
          | Some(id) => id->Belt.Int.toString->React.string
          },
        minmax: ("250px", "600px"),
      },
    ],
  },
  {
    label: "Metadata",
    cells: [
      {
        label: "Created On",
        view: d => d.createdAt->Js.Date.toLocaleDateString->React.string,
        minmax: ("150px", "300px"),
      },
      {
        label: "Established On",
        view: d =>
          d.establishedAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Announced On",
        view: d =>
          d.announcedAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Cancelled On",
        view: d =>
          d.cancelledAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
    ],
  },
]

module EditWorkplace = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~id: Uuid.t,
    ~detail: WorkplaceData.summary,
    ~setDetail,
  ) => {
    let (newsletterId, setNewsletterId) = React.useState(_ =>
      detail.newsletterId->Belt.Option.mapWithDefault("", Belt.Int.toString)
    )
    let (error, setError) = React.useState(() => None)

    let onSubmit = _ => {
      let parsedId = switch newsletterId {
      | "" => Ok(None)
      | s =>
        switch Belt.Int.fromString(s) {
        | Some(n) => Ok(Some(n))
        | None => Error("Newsletter ID must be a number")
        }
      }
      switch parsedId {
      | Error(msg) => setError(_ => Some(msg))
      | Ok(parsedNewsletterId) =>
        let body = Json.Encode.object([
          ("newsletter_id", Json.Encode.option(Json.Encode.int, parsedNewsletterId)),
        ])
        let req =
          api->Api.patchJson(
            ~path="/workplaces/" ++ Uuid.toString(id),
            ~decoder=WorkplaceData.Decode.summary,
            ~body,
          )
        req->Future.get(res => {
          switch res {
          | Ok(updated) => {
              setDetail(_ => RemoteData.Success(updated))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e->Api.showError))
          }
        })
      }
    }

    <Form onSubmit>
      <Form.TextField
        label="Newsletter ID"
        placeholder="42"
        value=newsletterId
        onInput={v => setNewsletterId(_ => v)}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Save")} </Button>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(msg) => <Message.Error> {React.string(msg)} </Message.Error>
      }}
    </Form>
  }
}

let editWorkplaceModal = (~api, ~modal, ~id, ~detail, ~setDetail): Modal.modalContent => {
  title: "Edit Workplace",
  content: <EditWorkplace api modal id detail setDetail />,
}

let statusButtons = (
  detail: Api.webData<WorkplaceData.summary>,
  ~onEstablish,
  ~onAnnounce,
  ~onCancel,
) => {
  switch detail {
  | Success(d) =>
    switch WorkplaceData.getStatus(d) {
    | Initial =>
      <Button.Panel>
        <Button variant=Button.Cta onClick=onEstablish> {React.string("Establish")} </Button>
      </Button.Panel>
    | Established =>
      <Button.Panel>
        <Button variant=Button.Cta onClick=onAnnounce> {React.string("Announce")} </Button>
        <Button variant=Button.Danger onClick=onCancel> {React.string("Cancel Workplace")} </Button>
      </Button.Panel>
    | Announced =>
      <Button.Panel>
        <Button variant=Button.Danger onClick=onCancel> {React.string("Cancel Workplace")} </Button>
      </Button.Panel>
    | Cancelled => React.null
    }
  | _ => React.null
  }
}

@react.component
let make = (~id: Uuid.t, ~api: Api.t, ~modal: Modal.Interface.t) => {
  let (detail, setDetail, _) =
    api->Hook.getData(
      ~path="/workplaces/" ++ Uuid.toString(id),
      ~decoder=WorkplaceData.Decode.summary,
    )

  let (error, setError) = React.useState(() => None)

  let doTransition = (path, _) => {
    let req =
      api->Api.patchJson(~path, ~decoder=WorkplaceData.Decode.summary, ~body=Json.Encode.object([]))
    req->Future.get(res => {
      switch res {
      | Ok(updated) => {
          setError(_ => None)
          setDetail(_ => RemoteData.Success(updated))
        }
      | Error(e) => setError(_ => Some(e))
      }
    })
  }

  let basePath = "/workplaces/" ++ Uuid.toString(id)

  let openEditModal = _ =>
    switch detail {
    | Success(d) =>
      Modal.Interface.openModal(modal, editWorkplaceModal(~api, ~modal, ~id, ~detail=d, ~setDetail))
    | _ => ()
    }

  <Page requireAnyRole=[ListWorkplaces] mainResource=detail>
    <header className={styles["header"]}>
      <h1 className={styles["title"]}>
        <span className={styles["titleText"]}>
          {React.string("Workplace ")}
          <span className={styles["titleId"]}>
            {switch detail {
            | Success(d) => d.id->Uuid.toString->React.string
            | _ => React.string("...")
            }}
          </span>
        </span>
        <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
          <Button onClick=openEditModal> {React.string("Edit")} </Button>
        </SessionContext.RequireRole>
      </h1>
      <div className={styles["headerNav"]}>
        <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
          <Page.BackButton name="workplaces" path="/workplaces" />
        </SessionContext.RequireRole>
        <SessionContext.RequireRole anyOf=[Session.ListWorkplaces]>
          <Page.BackButton name="workplace members" path={basePath ++ "/members"} />
        </SessionContext.RequireRole>
      </div>
    </header>
    <div className={styles["info"]}>
      <DataGrid layout data=detail />
    </div>
    {switch error {
    | None => React.null
    | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
    }}
    <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
      {statusButtons(
        detail,
        ~onEstablish=doTransition(basePath ++ "/establish"),
        ~onAnnounce=doTransition(basePath ++ "/announce"),
        ~onCancel=doTransition(basePath ++ "/cancel"),
      )}
    </SessionContext.RequireRole>
  </Page>
}
