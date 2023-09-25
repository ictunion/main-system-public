@module external styles: {..} = "./ApplicationDetail/styles.module.scss"

open Belt
open RemoteData
open Data

module Cell = {
  @react.component
  let make = (~label="", ~children: React.element) => {
    <div className={styles["cell"]}>
      {if label == "" {
        React.null
      } else {
        <span className={styles["label"]}> {React.string(label)} </span>
      }}
      <div> {children} </div>
    </div>
  }
}

module DataGrid = {
  type cell<'a> = {
    label: string,
    view: 'a => React.element,
    minmax: (string, string),
  }

  type row<'a> = {
    label: string,
    cells: array<cell<'a>>,
  }

  type t<'a> = array<row<'a>>

  let viewRow = (rowI, row: row<'a>, ~data: Api.webData<'a>) => {
    let gridTemplate: string = Array.map(row.cells, r => {
      let (min, max) = r.minmax
      "minmax(" ++ min ++ "," ++ max ++ ") "
    })->Js.String.concatMany("")

    let style =
      ReactDOM.Style.make()->ReactDOM.Style.unsafeAddProp("--grid-template-columns", gridTemplate)

    <div className={styles["rowWrap"]}>
      <h3 className={styles["rowLabel"]}> {React.string(row.label)} </h3>
      <div style key={Int.toString(rowI)} className={styles["row"]}>
        {row.cells
        ->Array.mapWithIndex((i, c) =>
          <Cell key={Int.toString(i)} label={c.label}>
            {switch data {
            | Success(d) => c.view(d)
            | _ => React.null
            }}
          </Cell>
        )
        ->React.array}
      </div>
    </div>
  }

  @react.component
  let make = (~layout: t<'a>, ~data: Api.webData<'a>) =>
    <div className={styles["dataGrid"]}>
      {layout->Array.mapWithIndex(viewRow(~data))->React.array}
    </div>
}

let viewOption = (o: option<'a>, view: 'a => React.element) => {
  switch o {
  | Some(v) => view(v)
  | None => React.null
  }
}

let layout: DataGrid.t<ApplicationData.detail> = [
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
        view: d => viewOption(d.dateOfBirth, a => a->Js.Date.toLocaleString->React.string),
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
  {
    label: "Employment",
    cells: [
      {
        label: "Company",
        view: d => viewOption(d.companyName, React.string),
        minmax: ("150px", "900px"),
      },
      {
        label: "Occupation",
        view: d => viewOption(d.occupation, React.string),
        minmax: ("150px", "665px"),
      },
    ],
  },
]

let metadataRows: array<RowBasedTable.row<ApplicationData.detail>> = [
  ("Localization", d => d.registrationLocal->Data.Local.toString->React.string),
  ("Source", d => viewOption(d.registrationSource, React.string)),
  ("IP Address", d => viewOption(d.registrationIp, React.string)),
  ("User Agent", d => viewOption(d.registrationUserAgent, React.string)),
]

let timeRows: array<RowBasedTable.row<ApplicationData.detail>> = [
  ("Created", d => d.createdAt->Js.Date.toLocaleString->React.string),
  (
    "Verification Sent",
    d => viewOption(d.verificationSentAt, a => a->Js.Date.toLocaleString->React.string),
  ),
  ("Email Verified", d => viewOption(d.verifiedAt, a => a->Js.Date.toLocaleString->React.string)),
  ("Rejected", d => viewOption(d.rejectedAt, a => a->Js.Date.toLocaleString->React.string)),
  ("Accepted", d => viewOption(d.acceptedAt, a => a->Js.Date.toLocaleString->React.string)),
]

type tabs =
  | Metadata
  | Checklist

@react.component
let make = (~id: string, ~api: Api.t) => {
  let (detail, _) =
    api->Hook.getData(~path="/applications/" ++ id, ~decoder=ApplicationData.Decode.detail)

  let tabHandlers = Tabbed.make(Metadata)

  <Page requireAnyRole=[ListApplications, ViewApplication]>
    <header>
      <h1 className={styles["title"]}>
        {React.string("Application ")}
        <span className={styles["titleId"]}>
          {switch detail {
          | Success(d) => d.id->Uuid.toString->React.string
          | _ => React.string("...")
          }}
        </span>
      </h1>
      <h2 className={styles["status"]}>
        {React.string("Status:")}
        <Chip.ApplicationStatus value={RemoteData.map(detail, ApplicationData.getStatus)} />
      </h2>
    </header>
    <div className={styles["personalInfo"]}>
      <DataGrid layout data=detail />
    </div>
    <Tabbed.Tabs>
      <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
      <Tabbed.Tab value=Checklist handlers=tabHandlers> {React.string("Checklist")} </Tabbed.Tab>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Metadata handlers=tabHandlers>
      <div className={styles["metadata"]}>
        <RowBasedTable rows=timeRows data=detail title=Some("Updates") />
        /* <RowBasedTable rows=metadataRows data=detail title=Some("Files") /> */
        <RowBasedTable rows=metadataRows data=detail title=Some("Metadata") />
      </div>
    </Tabbed.Content>
    <Tabbed.Content tab=Checklist handlers=tabHandlers> {React.string("TODO")} </Tabbed.Content>
  </Page>
}
