@module external styles: {..} = "./Applications/styles.module.scss"

/* TODO(turbomack):
 * Add callbacks to update basic stats after we get up to date data from individial tab loading
 */

open Data
open Belt

// TODO: This and Members.NewNote are pretty much copy pasted
// This should be abstracted and shared if the UX should be the same in both cases
module NewNote = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~refreshApplications,
    ~uuid: Uuid.t,
    ~initialNote: option<string>,
  ) => {
    let (note, setNote) = React.useState(_ =>
      switch initialNote {
      | None => ""
      | Some(v) => v
      }
    )
    let (error, setError) = React.useState(() => None)

    let onSubmit = _ => {
      let body: Js.Json.t = ApplicationData.Encode.newNote(note)
      let path = "/applications/" ++ Uuid.toString(uuid) ++ "/note"
      let req = api->Api.patchJson(~path, ~decoder=ApplicationData.Decode.summary, ~body)

      req->Future.get(res => {
        switch res {
        | Ok(_data) => {
            let _ = refreshApplications()
            Modal.Interface.closeModal(modal)
          }
        | Error(e) => setError(_ => Some(e))
        }
      })
    }

    <Form onSubmit>
      <Form.TextField
        label="Note"
        placeholder="...some text..."
        value=note
        onInput={updated => setNote(_ => updated)}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Update note")} </Button>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
    </Form>
  }
}

let newNoteModal = (~api, ~modal, ~refreshApplications, uuid, initialNote): Modal.modalContent => {
  title: "Update note",
  content: <NewNote api modal refreshApplications uuid initialNote />,
}

module Processing = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (processing, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications/processing",
        ~decoder=Json.Decode.array(ApplicationData.Decode.processingSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=processing
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
        },
        {
          name: "First Name",
          minMax: ("150px", "1fr"),
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
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
        {
          name: "Verified on",
          minMax: ("220px", "2fr"),
          view: r => React.string(r.verifiedAt->Js.Date.toLocaleDateString),
        },
      ]>
      {React.string("There are no applications in processiong.")}
    </DataTable>
  }
}

module Unverified = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (unverified, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications/unverified",
        ~decoder=Json.Decode.array(ApplicationData.Decode.unverifiedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=unverified
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
        },
        {
          name: "First Name",
          minMax: ("150px", "1fr"),
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
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
        {
          name: "Email Sent On",
          minMax: ("220px", "2fr"),
          view: r =>
            React.string(
              r.verificationSentAt->Option.mapWithDefault("NOT SENT!", Js.Date.toLocaleDateString),
            ),
        },
      ]>
      {React.string("There are no applications with pending verification.")}
    </DataTable>
  }
}

module Accepted = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications/accepted",
        ~decoder=Json.Decode.array(ApplicationData.Decode.acceptedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=all
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
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
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
        {
          name: "Accepted On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.acceptedAt->Js.Date.toLocaleDateString),
        },
      ]>
      {React.string("There are no accepted applications so far.")}
    </DataTable>
  }
}

module Rejected = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications/rejected",
        ~decoder=Json.Decode.array(ApplicationData.Decode.rejectedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=all
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
        },
        {
          name: "First Name",
          minMax: ("150px", "1fr"),
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
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
        {
          name: "Rejected On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.rejectedAt->Js.Date.toLocaleDateString),
        },
      ]>
      {React.string("There are no rejected applications so far.")}
    </DataTable>
  }
}

module Invalid = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications/invalid",
        ~decoder=Json.Decode.array(ApplicationData.Decode.invalidSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=all
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
        },
        {
          name: "First Name",
          minMax: ("150px", "1fr"),
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
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
        {
          name: "Invalidated On",
          minMax: ("170px", "1fr"),
          view: r => React.string(r.invalidatedAt->Js.Date.toLocaleDateString),
        },
      ]>
      {React.string("There are no invalidated applications so far.")}
    </DataTable>
  }
}

module All = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path="/applications",
        ~decoder=Json.Decode.array(ApplicationData.Decode.summary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshApplications, uuid, note))

    <DataTable
      data=all
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/applications/" ++ uuid} />,
        },
        {
          name: "First Name",
          minMax: ("150px", "1fr"),
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
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => r.companyName->View.option(React.string),
        },
        {
          name: "Language",
          minMax: ("125px", "1fr"),
          view: r => React.string(r.registrationLocal->Local.toString),
        },
      ]>
      {React.string("There are no applications so far.")}
    </DataTable>
  }
}

let urlToTab = (url: RescriptReactRouter.url): option<ApplicationData.status> => {
  open ApplicationData

  switch url.hash {
  | "all" => None
  | "unverified" => Some(Unverified)
  | "accepted" => Some(Accepted)
  | "rejected" => Some(Rejected)
  | "invalid" => Some(Invalid)
  | _ => Some(Processing)
  }
}

let tabToUrl = (tab: option<ApplicationData.status>): string => {
  open ApplicationData

  let hash = switch tab {
  | None => "all"
  | Some(Unverified) => "unverified"
  | Some(Accepted) => "accepted"
  | Some(Rejected) => "rejected"
  | Some(Processing) => "processing"
  | Some(Invalid) => "invalid"
  }

  "/applications#" ++ hash
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

  let (basicStats, _, _) =
    api->Hook.getData(
      ~path="/stats/applications/basic",
      ~decoder=StatsData.Applications.Decode.basic,
    )

  <Page requireAnyRole=[ListApplications]>
    <Page.Title> {React.string("Applications")} </Page.Title>
    {switch basicStats {
    | Success({unverified}) =>
      if unverified > 0 {
        <Message.Warning>
          <Message.Title> {React.string("Some applicants might be stuck..")} </Message.Title>
          <p> {React.string("Some applications did not pass email verification step yet.")} </p>
          <p>
            {React.string("
              This might be fine! Applicants can always verify email later.
              But it also can mean that some applicants didn't receive a verification email
              or missed the notice about email verification altogether.
              It might be a good idea to
            ")}
            <a onClick={_ => setActiveTab(_ => Some(ApplicationData.Unverified))}>
              {React.string("check the unverified applications")}
            </a>
            {React.string(" to make sure everything is fine.")}
          </p>
        </Message.Warning>
      } else {
        React.null
      }
    | _ => React.null
    }}
    <div className={styles["mainContent"]}>
      <Tabbed.Tabs>
        <Tabbed.Tab value={Some(ApplicationData.Processing)} handlers={tabHandlers}>
          <span> {React.string("Processing")} </span>
          <Chip.Count value={basicStats->RemoteData.map(r => r.processing)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Unverified)}
          handlers={tabHandlers}
          color=Some("var(--color5)")>
          {React.string("Pending Verification")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.unverified)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Accepted)} handlers={tabHandlers} color=Some("var(--color6)")>
          {React.string("Accepted")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.accepted)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Rejected)} handlers={tabHandlers} color=Some("var(--color7)")>
          {React.string("Rejected")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.rejected)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Invalid)} handlers={tabHandlers} color=Some("var(--color8)")>
          {React.string("Invalid")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.invalid)} />
        </Tabbed.Tab>
        <Tabbed.TabSpacer />
        <Tabbed.Tab value={None} handlers={tabHandlers} color=Some("var(--color1)")>
          {React.string("All")}
          <Chip.Count value={basicStats->RemoteData.map(StatsData.Applications.all)} />
        </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab={Some(ApplicationData.Processing)} handlers={tabHandlers}>
        <Processing api modal />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Unverified)} handlers={tabHandlers}>
        <Unverified api modal />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Accepted)} handlers={tabHandlers}>
        <Accepted api modal />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Rejected)} handlers={tabHandlers}>
        <Rejected api modal />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Invalid)} handlers={tabHandlers}>
        <Invalid api modal />
      </Tabbed.Content>
      <Tabbed.Content tab={None} handlers={tabHandlers}>
        <All api modal />
      </Tabbed.Content>
    </div>
  </Page>
}
