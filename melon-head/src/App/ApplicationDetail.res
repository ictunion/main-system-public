@module external styles: {..} = "./ApplicationDetail/styles.module.scss"

@send external focus: Dom.element => unit = "focus"
@scope("window") @val external window_open: (string, string) => unit = "open"

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

  let openFile = _ => {
    let fileUrl =
      api.host ++
      "/files/" ++
      file.id->Uuid.toString ++
      "?token=" ++
      api.keycloak->Keycloak.getToken

    window_open(fileUrl, "_blank")
  }

  <tr key={Uuid.toString(file.id)}>
    <td>
      <a onClick=openFile> {React.string(fileName)} </a>
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
          <Button variant=Button.Danger onClick=doReject> {React.string("Reject")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module Accept = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      let (memberNumber, setMemberNumber) = React.useState(_ => Some(""))

      // Api errors
      let (error, setError) = React.useState(() => None)

      let disabled = Form.MemberNumber.validate(memberNumber)->Option.isSome

      let doAccept = _ => {
        let body: Js.Json.t = Json.Encode.object([
          (
            "member_number",
            Json.Encode.option(Json.Encode.int, memberNumber->Option.flatMap(Int.fromString)),
          ),
        ])
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
      <Modal.Content>
        <p>
          {React.string("Accept application and create a ")}
          <strong> {React.string("new member")} </strong>
          {React.string("?")}
        </p>
        <Form.MemberNumber number=memberNumber setMemberNumber={num => setMemberNumber(_ => num)} />
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button disabled variant=Button.Cta onClick=doAccept> {React.string("Accept")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module Verify = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      let (error, setError) = React.useState(() => None)

      let doVerify = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/verify",
            ~decoder=ApplicationData.Decode.detail,
            ~body=Js.Json.null,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you shure you want to ")}
          <strong> {React.string("manualy verify the email address for this application")} </strong>
          {React.string("?")}
        </p>
        <p>
          {React.string(
            "Email address might not be valid or used. Doing this will bypass the verification process.",
          )}
        </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Danger onClick=doVerify> {React.string("Verify Anyway")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module ReSend = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~modal: Modal.Interface.t) => {
      let (error, setError) = React.useState(() => None)

      let doReSend = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.postJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/resend-email",
            ~decoder=Api.Decode.acceptedResponse,
            ~body=Js.Json.null,
          )

        req->Future.get(res => {
          switch res {
          | Ok(_) => {
              Modal.Interface.closeModal(modal)

              RescriptReactRouter.push("/applications")
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you shure you want to ")}
          <strong> {React.string("send verification email to applicant")} </strong>
          {React.string(" again?")}
        </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Cta onClick=doReSend> {React.string("Send Email")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module UnReject = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      let (error, setError) = React.useState(() => None)

      let doUnReject = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/unreject",
            ~decoder=ApplicationData.Decode.detail,
            ~body=Js.Json.null,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you shure you want to ")}
          <strong>
            {React.string("move this application back to processing to be re-evaluated")}
          </strong>
          {React.string(" again?")}
        </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Danger onClick=doUnReject>
            {React.string("Move Back to Processing")}
          </Button>
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

  let verifyModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Mark as Verified",
    content: <Verify api id setDetail modal />,
  }

  let resendModal = (~id, ~api, ~modal): Modal.modalContent => {
    title: "Re-Send Email Verification",
    content: <ReSend api id modal />,
  }

  let unRejectModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Move Application Back to Processing",
    content: <UnReject api id setDetail modal />,
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
    | ApplicationData.Unverified =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(rejectModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Danger>
          {React.string("Reject")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(verifyModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Danger>
          {React.string("Mark as Verified")}
        </Button>
        <Button
          onClick={_ => modal->Modal.Interface.openModal(resendModal(~id, ~api, ~modal))}
          variant=Button.Cta>
          {React.string("Re-send Email")}
        </Button>
      </Button.Panel>
    | ApplicationData.Processing =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(rejectModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Danger>
          {React.string("Reject")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(acceptModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Cta>
          {React.string("Accept")}
        </Button>
      </Button.Panel>
    | ApplicationData.Rejected =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(unRejectModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Danger>
          {React.string("Re-Evaluate")}
        </Button>
      </Button.Panel>
    | ApplicationData.Accepted => React.null
    }
  }
}

let viewMessage = (status, ~reject, ~resend) => {
  open ApplicationData

  switch status {
  | Success(Unverified) =>
    <Message.Warning>
      <Message.Title> {React.string("Email was not verified yet!")} </Message.Title>
      <p>
        {React.string(
          "This application is in `Verification Pending` status which means it was succefully submitted but applicant never clicked on verify link in the email.",
        )}
      </p>
      <p>
        <strong> {React.string("These are the recommended steps to take:")} </strong>
      </p>
      <ol>
        <li>
          {React.string(
            "Check when this application was submitted and when the confirmation email was sent using `Metadata` tab.",
          )}
        </li>
        <li>
          {React.string(
            "If application is very recent and there is an entry for when email was sent it might be good to just give applicant more time to verify it.",
          )}
        </li>
        <li>
          {React.string(
            "If we miss information about email being sent that is most likely an issue on our side. Try to ",
          )}
          <a onClick=resend> {React.string("resend the confirmation email")} </a>
          {React.string(" and check after a minute that it was sent successfully.")}
        </li>
        <li>
          {React.string(
            "If application is already few days old and there is still no confirmation might be good idea to look closer into it:",
          )}
          <ol>
            <li>
              {React.string("
                    You should review the information in the application and check if it's not a spam.
                    If it is a spam you should
                ")}
              <strong> {React.string("report that we got a spam")} </strong>
              {React.string(" and ")}
              <a onClick=reject> {React.string("reject this application")} </a>
              {React.string(" since it's ilegitimate.")}
            </li>
            <li>
              {React.string("
                    Check if this application is not a duplicate of some other application which is verified.
                    Applicant might had just found a mistake in this application and decided to create a new one.
                    If that's the case you should just
                ")}
              <a onClick=reject> {React.string("reject this version of application")} </a>
              {React.string(".")}
            </li>
            <li>
              {React.string("
                    If everything looks legitimate but you still don't see a verification you can try to
                ")}
              <a onClick=resend> {React.string("re-send the verification email again")} </a>
              {React.string("
                    if it looks like there is no problem with the email address.
                    If that doesn't help it might be a good idea to ")}
              <strong> {React.string("get in touch with the applicant if possible")} </strong>
              {React.string(" and try to sort thigs out over phone for example.")}
            </li>
            <li>
              {React.string("
                    If you're not able to get in touch with applicant and we still don't see any verification
                    there is probably nothing we can do but to
                ")}
              <a onClick=reject> {React.string("reject the application")} </a>
              {React.string(".")}
            </li>
          </ol>
        </li>
      </ol>
    </Message.Warning>
  | _ => React.null
  }
}

type tabs =
  | Metadata
  | Checklist
  | Files

@react.component
let make = (~id: Uuid.t, ~api: Api.t, ~modal: Modal.Interface.t) => {
  let (detail: Api.webData<ApplicationData.detail>, setDetail, _) =
    api->Hook.getData(
      ~path="/applications/" ++ Uuid.toString(id),
      ~decoder=ApplicationData.Decode.detail,
    )

  let (files, _, _) =
    api->Hook.getData(
      ~path="/applications/" ++ Uuid.toString(id) ++ "/files",
      ~decoder=Json.Decode.array(Data.Decode.file),
    )

  let tabHandlers = Tabbed.make(Files)

  let backToApplications = _ => {
    RescriptReactRouter.push("/applications")
  }

  let status = RemoteData.map(detail, ApplicationData.getStatus)

  <Page requireAnyRole=[ListApplications, ViewApplication]>
    <header className={styles["header"]}>
      <a className={styles["backBtn"]} onClick=backToApplications>
        {React.string("🠔 Back to applications")}
      </a>
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
        <Chip.ApplicationStatus value=status />
      </h2>
    </header>
    {viewMessage(
      status,
      ~reject=_ =>
        modal->Modal.Interface.openModal(Actions.rejectModal(~id, ~api, ~setDetail, ~modal)),
      ~resend=_ => modal->Modal.Interface.openModal(Actions.resendModal(~id, ~api, ~modal)),
    )}
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
