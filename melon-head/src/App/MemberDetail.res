@module external styles: {..} = "./MemberDetail/styles.module.scss"

open Data

let viewOption = (o: option<'a>, view: 'a => React.element) => {
  switch o {
  | Some(v) => view(v)
  | None => React.string("---")
  }
}

let layout: DataGrid.t<MemberData.detail> = [
  {
    label: "Membership",
    cells: [
      {
        label: "Member Number",
        view: d => Members.viewPaddedNumber(d.memberNumber),
        minmax: ("250px", "690px"),
      },
      {
        label: "Language",
        view: d => viewOption(d.language, React.string),
        minmax: ("200px", "200px"),
      },
      {
        label: "Application",
        view: d =>
          viewOption(d.applicationId, uuid =>
            <a onClick={_ => RescriptReactRouter.push("/applications/" ++ Uuid.toString(uuid))}>
              {React.string(uuid->Uuid.toString)}
            </a>
          ),
        minmax: ("250px", "655px"),
      },
    ],
  },
  {
    label: "Personal Information",
    cells: [
      {
        label: "First Name",
        view: d => viewOption(d.firstName, React.string),
        minmax: ("300px", "900px"),
      },
      {
        label: "Last Name",
        view: d => viewOption(d.lastName, React.string),
        minmax: ("225px", "665px"),
      },
      {
        label: "Date of Birth",
        view: d => viewOption(d.dateOfBirth, a => a->Js.Date.toLocaleDateString->React.string),
        minmax: ("250px", "250px"),
      },
    ],
  },
  {
    label: "Contacts",
    cells: [
      {
        label: "Email",
        view: d => viewOption(d.email, email => <Link.Email email />),
        minmax: ("300px", "900px"),
      },
      {
        label: "Phone Number",
        view: d => viewOption(d.phoneNumber, phoneNumber => <Link.Tel phoneNumber />),
        minmax: ("150px", "500px"),
      },
    ],
  },
  {
    label: "Address",
    cells: [
      {
        label: "Address",
        view: d => viewOption(d.address, React.string),
        minmax: ("450px", "900px"),
      },
      {label: "City", view: d => viewOption(d.city, React.string), minmax: ("150px", "500px")},
      {
        label: "Postal Code",
        view: d => viewOption(d.postalCode, React.string),
        minmax: ("150px", "150px"),
      },
    ],
  },
]

let timeRows: array<RowBasedTable.row<MemberData.detail>> = [
  ("Created", d => d.createdAt->Js.Date.toLocaleString->React.string),
  (
    "Onboarded at",
    d => viewOption(d.onboardingFinishAt, a => a->Js.Date.toLocaleString->React.string),
  ),
  ("Left at", d => viewOption(d.leftAt, a => a->Js.Date.toLocaleString->React.string)),
]

type tabs =
  | Metadata
  | Files

@react.component
let make = (~api, ~id) => {
  let (detail: Api.webData<MemberData.detail>, _setDetail, _) =
    api->Hook.getData(~path="/members/" ++ Uuid.toString(id), ~decoder=MemberData.Decode.detail)

  let status = RemoteData.map(detail, MemberData.getStatus)

  let tabHandlers = Tabbed.make(Metadata)

  <Page requireAnyRole=[ListMembers] mainResource=detail>
    <header className={styles["header"]}>
      <h1 className={styles["title"]}>
        {React.string("Member ")}
        <span className={styles["titleId"]}>
          {switch detail {
          | Success(d) => d.id->Uuid.toString->React.string
          | _ => React.string("...")
          }}
        </span>
      </h1>
      <Page.BackButton name="applications" path={status->RemoteData.toOption->Members.tabToUrl} />
      <h2 className={styles["status"]}>
        {React.string("Status:")}
        <Chip.MemberStatus value=status />
      </h2>
    </header>
    <DataGrid layout data=detail />
    <Tabbed.Tabs>
      <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Metadata handlers=tabHandlers>
      <div className={styles["metadata"]}>
        <RowBasedTable rows=timeRows data=detail title=Some("Updates") />
      </div>
    </Tabbed.Content>
  </Page>
}
