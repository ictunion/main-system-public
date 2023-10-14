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
      ]
    />
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
          minMax: ("150px", "2fr"),
          view: r =>
            React.string(
              r.verificationSentAt->Option.mapWithDefault("NOT SENT!", Js.Date.toLocaleDateString),
            ),
        },
      ]
    />
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
      ]
    />
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
      ]
    />
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
      ]
    />
  }
}

@react.component
let make = (~api: Api.t) => {
  let tabHandlers = Tabbed.make(Some(ApplicationData.Processing))
  let (_, setActiveTab) = tabHandlers

  let (basicStats, _, _) = api->Hook.getData(~path="/stats/basic", ~decoder=StatsData.Decode.basic)

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
              This might be fine since applicants can always verify email later.
              But it also can be the case that some applicants didn't receive the email
              or missed the notice about verification altogether.
              It might be a good idea to
            ")}
            <a onClick={_ => setActiveTab(_ => Some(ApplicationData.Unverified))}>
              {React.string("check the unverified applications")}
            </a>
            {React.string(" and make sure this is not the case.")}
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
          <Chip.Count value={basicStats->RemoteData.map(StatsData.all)} />
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
