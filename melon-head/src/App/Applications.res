@module external styles: {..} = "./Applications/styles.module.scss"

/* TODO(turbomack):
 * Add callbacks to update basic stats after we get up to date data from individial tab loading
 */

open Data

/* Admin: every application. Workplace: only applications assigned to the
 * caller's workplace (executive committee view), without notes and city. */
type scope = Admin | Workplace

let apiPrefix = scope =>
  switch scope {
  | Admin => "/applications"
  | Workplace => "/applications/workplace"
  }

/* The workplace listing comes without note and city (redacted server-side) */
let forScope = (columns: array<DataTable.column<'t>>, scope) =>
  switch scope {
  | Admin => columns
  | Workplace => columns->Array.filter(c => c.name != "Note" && c.name != "City")
  }

let viewLastName = (~lastName, ~approvedByRepsAt) =>
  <span className={styles["withApproval"]}>
    {lastName->View.option(React.string)}
    {approvedByRepsAt->Option.mapOr(React.null, at =>
      <span
        className={styles["approved"]}
        title={"Approved by workplace representatives at " ++ at->Date.toLocaleString}
      >
        {React.string("✓")}
      </span>
    )}
  </span>

module Processing = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (processing, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope) ++ "/processing"},
        ~decoder=Json.Decode.array(ApplicationData.Decode.processingSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.processingSummary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
        view: r => React.string(r.verifiedAt->Date.toLocaleDateString),
      },
    ]

    <DataTable data=processing columns={columns->forScope(scope)}>
      {React.string("There are no applications in processiong.")}
    </DataTable>
  }
}

module Unverified = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (unverified, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope) ++ "/unverified"},
        ~decoder=Json.Decode.array(ApplicationData.Decode.unverifiedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.unverifiedSummary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
          React.string(r.verificationSentAt->Option.mapOr("NOT SENT!", Date.toLocaleDateString)),
      },
    ]

    <DataTable data=unverified columns={columns->forScope(scope)}>
      {React.string("There are no applications with pending verification.")}
    </DataTable>
  }
}

module Accepted = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope) ++ "/accepted"},
        ~decoder=Json.Decode.array(ApplicationData.Decode.acceptedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.acceptedSummary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
        view: r => React.string(r.acceptedAt->Date.toLocaleDateString),
      },
    ]

    <DataTable data=all columns={columns->forScope(scope)}>
      {React.string("There are no accepted applications so far.")}
    </DataTable>
  }
}

module Rejected = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope) ++ "/rejected"},
        ~decoder=Json.Decode.array(ApplicationData.Decode.rejectedSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.rejectedSummary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
        view: r => React.string(r.rejectedAt->Date.toLocaleDateString),
      },
    ]

    <DataTable data=all columns={columns->forScope(scope)}>
      {React.string("There are no rejected applications so far.")}
    </DataTable>
  }
}

module Invalid = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope) ++ "/invalid"},
        ~decoder=Json.Decode.array(ApplicationData.Decode.invalidSummary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.invalidSummary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
        view: r => React.string(r.invalidatedAt->Date.toLocaleDateString),
      },
    ]

    <DataTable data=all columns={columns->forScope(scope)}>
      {React.string("There are no invalidated applications so far.")}
    </DataTable>
  }
}

module All = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope: scope) => {
    let (all, _, refreshApplications) =
      api->Hook.getData(
        ~path={apiPrefix(scope)},
        ~decoder=Json.Decode.array(ApplicationData.Decode.summary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        NewNote.Application.modal(~api, ~modal, ~refresh=refreshApplications, uuid, note),
      )

    let columns: array<DataTable.column<ApplicationData.summary>> = [
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
        view: r => viewLastName(~lastName=r.lastName, ~approvedByRepsAt=r.approvedByRepsAt),
      },
      {
        name: "Note",
        minMax: ("250px", "10fr"),
        view: r =>
          <a onClick={_ => openNewNoteModal(r.id, r.note)}>
            {
              let note = if Option.getOr(r.note, "Add note") == "" {
                "Add note"
              } else {
                Option.getOr(r.note, "Add note")
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
    ]

    <DataTable data=all columns={columns->forScope(scope)}>
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

let pageUrl = scope =>
  switch scope {
  | Admin => "/applications"
  | Workplace => "/my-workplace-applications"
  }

let tabToUrl = (~scope=Admin, tab: option<ApplicationData.status>): string => {
  open ApplicationData

  let hash = switch tab {
  | None => "all"
  | Some(Unverified) => "unverified"
  | Some(Accepted) => "accepted"
  | Some(Rejected) => "rejected"
  | Some(Processing) => "processing"
  | Some(Invalid) => "invalid"
  }

  pageUrl(scope) ++ "#" ++ hash
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t, ~scope=Admin) => {
  let (activeTab, setActiveTab_) = React.useState(_ =>
    RescriptReactRouter.dangerouslyGetInitialUrl()->urlToTab
  )

  let _ = RescriptReactRouter.watchUrl(url => {
    setActiveTab_(_ => urlToTab(url))
  })

  let setActiveTab = f => {
    let newTab = f(activeTab)
    RescriptReactRouter.push(tabToUrl(~scope, newTab))
  }

  let tabHandlers = (activeTab, setActiveTab)

  let (basicStats, _, _) = api->Hook.getData(
    ~path=switch scope {
    | Admin => "/stats/applications/basic"
    | Workplace => "/stats/applications/workplace/basic"
    },
    ~decoder=StatsData.Applications.Decode.basic,
  )

  let (workplace, _, _) =
    api->Hook.getData(
      ~enabled=scope == Workplace,
      ~path="/workplaces/mine",
      ~decoder=WorkplaceData.Decode.summary,
    )

  <Page
    requireAnyRole={switch scope {
    | Admin => [ListApplications]
    | Workplace => [ListOwnWorkplaceApplications]
    }}
  >
    <Page.Title>
      {React.string(
        switch scope {
        | Admin => "Applications"
        | Workplace =>
          switch workplace {
          | Success(w) => "Applications of " ++ w.name
          | _ => "Workplace Applications"
          }
        },
      )}
    </Page.Title>
    {switch (scope, basicStats) {
    | (Admin, Success({unverified})) =>
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
          color=Some("var(--color5)")
        >
          {React.string("Pending Verification")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.unverified)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Accepted)} handlers={tabHandlers} color=Some("var(--color6)")
        >
          {React.string("Accepted")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.accepted)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Rejected)} handlers={tabHandlers} color=Some("var(--color7)")
        >
          {React.string("Rejected")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.rejected)} />
        </Tabbed.Tab>
        <Tabbed.Tab
          value={Some(ApplicationData.Invalid)} handlers={tabHandlers} color=Some("var(--color8)")
        >
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
        <Processing api modal scope />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Unverified)} handlers={tabHandlers}>
        <Unverified api modal scope />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Accepted)} handlers={tabHandlers}>
        <Accepted api modal scope />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Rejected)} handlers={tabHandlers}>
        <Rejected api modal scope />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Invalid)} handlers={tabHandlers}>
        <Invalid api modal scope />
      </Tabbed.Content>
      <Tabbed.Content tab={None} handlers={tabHandlers}>
        <All api modal scope />
      </Tabbed.Content>
    </div>
  </Page>
}
