open Data
open Belt

@module external styles: {..} = "./Applications/styles.module.scss"

type applicationsTab =
  | Processing
  | Unverified
  | Accepted
  | Rejected

module Data = {
  type processingSummary = {
    id: Uuid.t,
    email: option<Email.t>,
    firstName: option<string>,
    lastName: option<string>,
    phoneNumber: option<PhoneNumber.t>,
    city: option<string>,
    companyName: option<string>,
    registrationLocal: Local.t,
    createdAt: Js.Date.t,
    verifiedAt: Js.Date.t,
  }

  type unverifiedSummary = {
    id: Uuid.t,
    email: option<Email.t>,
    firstName: option<string>,
    lastName: option<string>,
    phoneNumber: option<PhoneNumber.t>,
    city: option<string>,
    companyName: option<string>,
    registrationLocal: Local.t,
    createdAt: Js.Date.t,
    verificationSentAt: option<Js.Date.t>,
  }

  module Decode = {
    open Json.Decode

    let processingSummary = object(field => {
      id: field.required(. "id", Uuid.decode),
      email: field.required(. "email", option(Email.decode)),
      firstName: field.required(. "first_name", option(string)),
      lastName: field.required(. "last_name", option(string)),
      phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
      city: field.required(. "city", option(string)),
      companyName: field.required(. "company_name", option(string)),
      registrationLocal: field.required(. "registration_local", Local.decode),
      createdAt: field.required(. "created_at", date),
      verifiedAt: field.required(. "confirmed_at", date),
    })

    let unverifiedSummary = object(field => {
      id: field.required(. "id", Uuid.decode),
      email: field.required(. "email", option(Email.decode)),
      firstName: field.required(. "first_name", option(string)),
      lastName: field.required(. "last_name", option(string)),
      phoneNumber: field.required(. "phone_number", option(PhoneNumber.decode)),
      city: field.required(. "city", option(string)),
      companyName: field.required(. "company_name", option(string)),
      registrationLocal: field.required(. "registration_local", Local.decode),
      createdAt: field.required(. "created_at", date),
      verificationSentAt: field.required(. "verification_sent_at", option(date)),
    })
  }
}

@react.component
let make = (~api: Api.t) => {
  let tabHandlers = Tabbed.make(Processing)

  let (basicStats, _) = api->Hook.getData(~path="/stats/basic", ~decoder=Stats.Decode.basic)
  let (processing, _) =
    api->Hook.getData(
      ~path="/applications/processing",
      ~decoder=Json.Decode.array(Data.Decode.processingSummary),
    )

  let (unverified, _) =
    api->Hook.getData(
      ~path="/applications/unverified",
      ~decoder=Json.Decode.array(Data.Decode.unverifiedSummary),
    )

  <Page requireAnyRole=[ListApplications]>
    <Page.Title> {React.string("Applications")} </Page.Title>
    <div className={styles["main-content"]}>
      <Tabbed.Tabs>
        <Tabbed.Tab value={Processing} handlers={tabHandlers}>
          <span> {React.string("Processing")} </span>
          <Chip.Count value={basicStats->RemoteData.map(r => r.processing)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Unverified} handlers={tabHandlers} color=Some("#a63ded")>
          {React.string("Pending Verification")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.unverified)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Accepted} handlers={tabHandlers} color=Some("#00c49f")>
          {React.string("Accepted")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.accepted)} />
        </Tabbed.Tab>
        <Tabbed.Tab value={Rejected} handlers={tabHandlers} color=Some("#ef562e")>
          {React.string("Rejected")}
          <Chip.Count value={basicStats->RemoteData.map(r => r.rejected)} />
        </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab={Processing} handlers={tabHandlers}>
        <DataTable
          data={processing}
          columns={[
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
              minMax: ("200px", "2fr"),
              view: r =>
                r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
            },
            {
              name: "Phone",
              minMax: ("250px", "2fr"),
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
              name: "Verified at",
              minMax: ("150px", "2fr"),
              view: r => React.string(r.verifiedAt->Js.Date.toLocaleString),
            },
          ]}
        />
      </Tabbed.Content>
      <Tabbed.Content tab={Unverified} handlers={tabHandlers}>
        <DataTable
          data={unverified}
          columns={[
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
              minMax: ("200px", "2fr"),
              view: r =>
                r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
            },
            {
              name: "Phone",
              minMax: ("150px", "2fr"),
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
              name: "Email sent at",
              minMax: ("150px", "2fr"),
              view: r =>
                React.string(
                  r.verificationSentAt->Option.mapWithDefault("NOT SENT!", Js.Date.toLocaleString),
                ),
            },
          ]}
        />
      </Tabbed.Content>
    </div>
  </Page>
}
