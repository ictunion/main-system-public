@module external styles: {..} = "./Applications/styles.module.scss"

/* TODO(turbomack):
 * Add callbacks to update basic stats after we get up to date data from individial tab loading
 */

open Data
open Belt

module Processing = {
  @react.component
  let make = (~api: Api.t) => {
    let (processing, _, _) =
      api->Hook.getData(
        ~path="/applications/processing",
        ~decoder=Json.Decode.array(ApplicationData.Decode.processingSummary),
      )

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
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.companyName->Option.getWithDefault("--")),
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
  let make = (~api: Api.t) => {
    let (unverified, _, _) =
      api->Hook.getData(
        ~path="/applications/unverified",
        ~decoder=Json.Decode.array(ApplicationData.Decode.unverifiedSummary),
      )

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
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.companyName->Option.getWithDefault("--")),
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
  let make = (~api: Api.t) => {
    let (all, _, _) =
      api->Hook.getData(
        ~path="/applications/accepted",
        ~decoder=Json.Decode.array(ApplicationData.Decode.acceptedSummary),
      )

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
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.companyName->Option.getWithDefault("--")),
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
  let make = (~api: Api.t) => {
    let (all, _, _) =
      api->Hook.getData(
        ~path="/applications/rejected",
        ~decoder=Json.Decode.array(ApplicationData.Decode.rejectedSummary),
      )

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
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.companyName->Option.getWithDefault("--")),
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

module All = {
  @react.component
  let make = (~api: Api.t) => {
    let (all, _, _) =
      api->Hook.getData(
        ~path="/applications",
        ~decoder=Json.Decode.array(ApplicationData.Decode.summary),
      )

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
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "2fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Company Name",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.companyName->Option.getWithDefault("--")),
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
  }

  "/applications#" ++ hash
}

@react.component
let make = (~api: Api.t) => {
  let (activeTab, setActiveTab) = React.useState(_ =>
    RescriptReactRouter.dangerouslyGetInitialUrl()->urlToTab
  )

  let _ = RescriptReactRouter.watchUrl(url => {
    setActiveTab(_ => urlToTab(url))
  })

  let tabHandlers = (
    activeTab,
    f => {
      let newTab = f(activeTab)
      RescriptReactRouter.push(tabToUrl(newTab))
    },
  )

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
        <Tabbed.TabSpacer />
        <Tabbed.Tab value={None} handlers={tabHandlers} color=Some("var(--color1)")>
          {React.string("All")}
          <Chip.Count value={basicStats->RemoteData.map(StatsData.Applications.all)} />
        </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab={Some(ApplicationData.Processing)} handlers={tabHandlers}>
        <Processing api />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Unverified)} handlers={tabHandlers}>
        <Unverified api />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Accepted)} handlers={tabHandlers}>
        <Accepted api />
      </Tabbed.Content>
      <Tabbed.Content tab={Some(ApplicationData.Rejected)} handlers={tabHandlers}>
        <Rejected api />
      </Tabbed.Content>
      <Tabbed.Content tab={None} handlers={tabHandlers}>
        <All api />
      </Tabbed.Content>
    </div>
  </Page>
}
