@module external styles: {..} = "./ApplicationDetail/styles.module.scss"

@send external focus: Dom.element => unit = "focus"

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

    <div key={Int.toString(rowI)} className={styles["rowWrap"]}>
      {if row.label != "" {
        <h3 className={styles["rowLabel"]}> {React.string(row.label)} </h3>
      } else {
        React.null
      }}
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

let viewSignature = (~api: Api.t, signature: option<Data.file>) => {
  <div className={styles["signature"]}>
    {switch signature {
    | Some(file) => {
        let src =
          api.host ++
          "/files/" ++
          file.id->Uuid.toString ++
          "?token=" ++
          api.keycloak->Keycloak.getToken
        <img alt="signature" src />
      }
    | None => React.string("Not Found")
    }}
  </div>
}

let viewFile = (~api: Api.t, file) => {
  let fileName = file.name ++ "." ++ file.fileType
  let fileUrl =
    api.host ++ "/files/" ++ file.id->Uuid.toString ++ "?token=" ++ api.keycloak->Keycloak.getToken

  <tr key={file.id->Uuid.toString}>
    <td>
      <a href=fileUrl target="_blank"> {React.string(fileName)} </a>
    </td>
    <td> {file.createdAt->Js.Date.toLocaleString->React.string} </td>
  </tr>
}

module Actions = {
  module Reject = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      let (error, setError) = React.useState(() => None)

      let doReject = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.deleteJson(
            ~path="/applications/" ++ Uuid.toString(id),
            ~decoder=ApplicationData.Decode.detail,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              // Technically, we probably don't need to be setting this date if we're going to route away anyway
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)

              // open list
              RescriptReactRouter.push("/applications")
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you shure you want to ")}
          <strong> {React.string("reject the application")} </strong>
          {React.string("?")}
        </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button btnType=Button.Danger onClick=doReject> {React.string("Reject")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module Accept = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      // DOM node of text input
      let inputEl = React.useRef(Js.Nullable.null)

      // These two valies cold be also collapesed to option type
      // which might be nicer in terms of types but a bit less practical
      // for binding to component properties
      let (hasCustomNumber, setHasCustomNumber) = React.useState(() => true)
      let (memberNumber, setMemberNumber) = React.useState(() => "")
      let toggleHasCustomNumber = _ => setHasCustomNumber(v => !v)

      // Validations
      let inputEmpyErr = Some("Member number can't be empty.")
      let (validationErr, setValidationErr) = React.useState(() => inputEmpyErr)

      // Api errors
      let (error, setError) = React.useState(() => None)

      let doAccept = _ => {
        open Json
        let num = if hasCustomNumber {
          Int.fromString(memberNumber)
        } else {
          None
        }
        let body: Js.Json.t = Encode.object([("member_number", Encode.option(Encode.int, num))])
        let path = "/applications/" ++ Uuid.toString(id) ++ "/accept"

        let req = api->Api.postJson(~path, ~decoder=ApplicationData.Decode.detail, ~body)

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              // Technically, we probably don't need to be setting this date if we're going to route away anyway
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
              RescriptReactRouter.push("/applications")
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      React.useEffect0(() => {
        inputEl.current->Js.Nullable.toOption->Belt.Option.forEach(input => input->focus)
        None
      })

      let onInput = (event: JsxEvent.Form.t) => {
        let newVal = ReactEvent.Form.currentTarget(event)["value"]
        setMemberNumber(_ => newVal)

        // validate input
        switch newVal->Int.fromString {
        | None =>
          if newVal == "" {
            setValidationErr(_ => inputEmpyErr)
          } else {
            setValidationErr(_ => Some("Expects number, got `" ++ newVal ++ "`."))
          }
        | Some(i) =>
          if i <= 0 {
            setValidationErr(_ => Some("Member number must be positive number (greater than `0`)."))
          } else {
            setValidationErr(_ => None)
          }
        }
      }

      let disabled = if hasCustomNumber {
        switch validationErr {
        | Some(_) => true
        | None => false
        }
      } else {
        false
      }

      <Modal.Content>
        <p>
          {React.string("Accept application and create a ")}
          <strong> {React.string("new member")} </strong>
          {React.string("?")}
        </p>
        <div className={styles["memberNumberAuto"]}>
          <label>
            <input type_="checkbox" checked={!hasCustomNumber} onChange=toggleHasCustomNumber />
            <strong> {React.string("Automatically assign member number")} </strong>
          </label>
          {if hasCustomNumber {
            React.null
          } else {
            <Message.Info>
              <strong>
                {React.string(
                  "New members will be automatically assigned a number that is one greater (+1) than the highest current existing member number.",
                )}
              </strong>
            </Message.Info>
          }}
        </div>
        {if hasCustomNumber {
          <form className={styles["memberNumberForm"]}>
            <label className={styles["memberNumberField"]}>
              {React.string("Member number:")}
              <br />
              <input
                ref={ReactDOM.Ref.domRef(inputEl)} value=memberNumber onInput placeholder="42"
              />
            </label>
            {validationErr->viewOption(str => {
              <Message.Error> {React.string(str)} </Message.Error>
            })}
          </form>
        } else {
          React.null
        }}
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button disabled btnType=Button.Cta onClick=doAccept> {React.string("Accept")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  let rejectModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Reject Application",
    content: <Reject api id setDetail modal />,
  }

  let acceptModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Accept Application",
    content: <Accept api id setDetail modal />,
  }

  @react.component
  let make = (
    ~id: Uuid.t,
    ~api: Api.t,
    ~status: ApplicationData.status,
    ~modal: Modal.Interface.t,
    ~setDetail,
  ) => {
    switch status {
    | ApplicationData.Processing =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(rejectModal(~id, ~api, ~setDetail, ~modal))}
          btnType=Button.Danger>
          {React.string("Reject")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(acceptModal(~id, ~api, ~setDetail, ~modal))}
          btnType=Button.Cta>
          {React.string("Accept")}
        </Button>
      </Button.Panel>
    | _ => React.null
    }
  }
}

type tabs =
  | Metadata
  | Checklist
  | Files

@react.component
let make = (~id: Uuid.t, ~api: Api.t, ~modal: Modal.Interface.t) => {
  let (detail: Api.webData<ApplicationData.detail>, setDetail) =
    api->Hook.getData(
      ~path="/applications/" ++ Uuid.toString(id),
      ~decoder=ApplicationData.Decode.detail,
    )

  let (files, _) =
    api->Hook.getData(
      ~path="/applications/" ++ Uuid.toString(id) ++ "/files",
      ~decoder=Json.Decode.array(Data.Decode.file),
    )

  let tabHandlers = Tabbed.make(Files)

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
      <Tabbed.Tab value=Files handlers=tabHandlers> {React.string("Files")} </Tabbed.Tab>
      <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
      <Tabbed.Tab value=Checklist handlers=tabHandlers> {React.string("Checklist")} </Tabbed.Tab>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Files handlers=tabHandlers>
      <DataGrid
        data=files
        layout={[
          {
            label: "",
            cells: [
              {
                label: "Signature",
                minmax: ("150px", "600px"),
                view: files =>
                  files
                  ->Array.getByU((. f) => {
                    f.name == "signature" && f.fileType == "png"
                  })
                  ->viewSignature(~api),
              },
              {
                label: "All Files",
                minmax: ("150px", "965px"),
                view: files =>
                  <table className={styles["filesTable"]}>
                    <thead>
                      <tr>
                        <td> {React.string("File Name")} </td>
                        <td> {React.string("Created at")} </td>
                      </tr>
                    </thead>
                    <tbody> {files->Array.map(viewFile(~api))->React.array} </tbody>
                  </table>,
              },
            ],
          },
        ]}
      />
    </Tabbed.Content>
    <Tabbed.Content tab=Metadata handlers=tabHandlers>
      <div className={styles["metadata"]}>
        <RowBasedTable rows=timeRows data=detail title=Some("Updates") />
        <RowBasedTable rows=metadataRows data=detail title=Some("Metadata") />
      </div>
    </Tabbed.Content>
    <Tabbed.Content tab=Checklist handlers=tabHandlers> {React.string("TODO")} </Tabbed.Content>
    {switch RemoteData.map(detail, ApplicationData.getStatus) {
    | Success(status) => <Actions id api modal setDetail status />
    | _ => React.null
    }}
  </Page>
}
