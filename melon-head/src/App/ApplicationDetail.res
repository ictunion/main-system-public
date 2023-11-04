@module external styles: {..} = "./ApplicationDetail/styles.module.scss"

@send external focus: Dom.element => unit = "focus"
@scope("window") @val external window_open: (string, string) => unit = "open"

open Belt
open RemoteData
open Data

let layout: DataGrid.t<ApplicationData.detail> = [
  {
    label: "Personal Information",
    cells: [
      {
        label: "First Name",
        view: d => View.option(d.firstName, React.string),
        minmax: ("300px", "900px"),
      },
      {
        label: "Last Name",
        view: d => View.option(d.lastName, React.string),
        minmax: ("225px", "665px"),
      },
      {
        label: "Date of Birth",
        view: d => View.option(d.dateOfBirth, a => a->Js.Date.toLocaleDateString->React.string),
        minmax: ("250px", "250px"),
      },
    ],
  },
  {
    label: "Contacts",
    cells: [
      {
        label: "Email",
        view: d => View.option(d.email, email => <Link.Email email />),
        minmax: ("300px", "900px"),
      },
      {
        label: "Phone Number",
        view: d => View.option(d.phoneNumber, phoneNumber => <Link.Tel phoneNumber />),
        minmax: ("150px", "500px"),
      },
    ],
  },
  {
    label: "Address",
    cells: [
      {
        label: "Address",
        view: d => View.option(d.address, React.string),
        minmax: ("450px", "900px"),
      },
      {label: "City", view: d => View.option(d.city, React.string), minmax: ("150px", "500px")},
      {
        label: "Postal Code",
        view: d => View.option(d.postalCode, React.string),
        minmax: ("150px", "150px"),
      },
    ],
  },
  {
    label: "Employment",
    cells: [
      {
        label: "Company",
        view: d => View.option(d.companyName, React.string),
        minmax: ("150px", "900px"),
      },
      {
        label: "Occupation",
        view: d => View.option(d.occupation, React.string),
        minmax: ("150px", "665px"),
      },
    ],
  },
]

let metadataRows: array<RowBasedTable.row<ApplicationData.detail>> = [
  ("Localization", d => d.registrationLocal->Data.Local.toString->React.string),
  ("Source", d => View.option(d.registrationSource, str => <code> {React.string(str)} </code>)),
  ("IP Address", d => View.option(d.registrationIp, React.string)),
  ("User Agent", d => View.option(d.registrationUserAgent, React.string)),
]

let timeRows: array<RowBasedTable.row<ApplicationData.detail>> = [
  ("Created", d => d.createdAt->Js.Date.toLocaleString->React.string),
  (
    "Verification Sent",
    d => View.option(d.verificationSentAt, a => a->Js.Date.toLocaleString->React.string),
  ),
  ("Email Verified", d => View.option(d.verifiedAt, a => a->Js.Date.toLocaleString->React.string)),
  ("Rejected", d => View.option(d.rejectedAt, a => a->Js.Date.toLocaleString->React.string)),
  ("Invalidated", d => View.option(d.invalidatedAt, a => a->Js.Date.toLocaleString->React.string)),
  ("Accepted", d => View.option(d.acceptedAt, a => a->Js.Date.toLocaleString->React.string)),
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
    let make = (
      ~api: Api.t,
      ~id: Uuid.t,
      ~setDetail,
      ~modal: Modal.Interface.t,
      ~openApplications,
    ) => {
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

              openApplications()
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

  module Invalidate = {
    @react.component
    let make = (
      ~api: Api.t,
      ~id: Uuid.t,
      ~setDetail,
      ~modal: Modal.Interface.t,
      ~openApplications,
    ) => {
      let (error, setError) = React.useState(() => None)

      let doReject = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/invalidate",
            ~decoder=ApplicationData.Decode.detail,
            ~body=Js.Json.null,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              // Technically, we probably don't need to be setting this date if we're going to route away anyway
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)

              openApplications()
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you shure you want to ")}
          <strong> {React.string("invalidate the application")} </strong>
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
          <Button variant=Button.Careful onClick=doReject> {React.string("Invalidate")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  module Accept = {
    @react.component
    let make = (
      ~api: Api.t,
      ~id: Uuid.t,
      ~setDetail,
      ~modal: Modal.Interface.t,
      ~openApplications,
    ) => {
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

              openApplications()
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
    let make = (~api: Api.t, ~id: Uuid.t, ~modal: Modal.Interface.t, ~openApplications) => {
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

              openApplications()
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

  module UnInvalidate = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~setDetail, ~modal: Modal.Interface.t) => {
      let (error, setError) = React.useState(() => None)

      let doUnReject = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/uninvalidate",
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

  module HardDelete = {
    @react.component
    let make = (~api: Api.t, ~id: Uuid.t, ~modal: Modal.Interface.t, ~openApplications) => {
      let (error, setError) = React.useState(() => None)

      let doHardDelete = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.deleteJson(
            ~path="/applications/" ++ Uuid.toString(id) ++ "/hard",
            ~decoder=Api.Decode.acceptedResponse,
          )

        req->Future.get(res => {
          switch res {
          | Ok(_) => {
              Modal.Interface.closeModal(modal)

              openApplications()
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p>
          {React.string("Are you sure you want to ")}
          <strong> {React.string("permenently remove")} </strong>
          {React.string(" this application?")}
        </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => Modal.Interface.closeModal(modal)}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Danger onClick=doHardDelete>
            {React.string("HARD DELETE")}
          </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  let rejectModal = (~id, ~api, ~setDetail, ~modal, ~openApplications): Modal.modalContent => {
    title: "Reject Application",
    content: <Reject api id setDetail modal openApplications />,
  }

  let invalidateModal = (~id, ~api, ~setDetail, ~modal, ~openApplications): Modal.modalContent => {
    title: "Invalidate Application",
    content: <Invalidate api id setDetail modal openApplications />,
  }

  let acceptModal = (~id, ~api, ~setDetail, ~modal, ~openApplications): Modal.modalContent => {
    title: "Accept Application",
    content: <Accept api id setDetail modal openApplications />,
  }

  let verifyModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Mark as Verified",
    content: <Verify api id setDetail modal />,
  }

  let resendModal = (~id, ~api, ~modal, ~openApplications): Modal.modalContent => {
    title: "Re-Send Email Verification",
    content: <ReSend api id modal openApplications />,
  }

  let unRejectModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Move Application Back to Processing",
    content: <UnReject api id setDetail modal />,
  }

  let unInvalidateModal = (~id, ~api, ~setDetail, ~modal): Modal.modalContent => {
    title: "Move Application Back to Processing",
    content: <UnInvalidate api id setDetail modal />,
  }

  let hardDeleteModal = (~id, ~api, ~modal, ~openApplications): Modal.modalContent => {
    title: "Remove PERMANENTLY",
    content: <HardDelete api id modal openApplications />,
  }

  @react.component
  let make = (
    ~id: Uuid.t,
    ~api: Api.t,
    ~status: ApplicationData.status,
    ~modal: Modal.Interface.t,
    ~setDetail,
    ~openApplications,
  ) => {
    switch status {
    | ApplicationData.Unverified =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(
              invalidateModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
            )}
          variant=Button.Careful>
          {React.string("Invalidate")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(verifyModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Careful>
          {React.string("Mark as Verified")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(
              rejectModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
            )}
          variant=Button.Danger>
          {React.string("Reject")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(resendModal(~id, ~api, ~modal, ~openApplications))}
          variant=Button.Normal>
          {React.string("Re-send Email")}
        </Button>
      </Button.Panel>
    | ApplicationData.Processing =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(
              invalidateModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
            )}
          variant=Button.Careful>
          {React.string("Invalidate")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(
              rejectModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
            )}
          variant=Button.Danger>
          {React.string("Reject")}
        </Button>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(
              acceptModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
            )}
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
    | ApplicationData.Invalid =>
      <Button.Panel>
        <Button
          onClick={_ =>
            modal->Modal.Interface.openModal(unInvalidateModal(~id, ~api, ~setDetail, ~modal))}
          variant=Button.Danger>
          {React.string("Re-Evaluate")}
        </Button>
        <SessionContext.RequireRole anyOf=[Session.SuperPowers]>
          <Button
            onClick={_ =>
              modal->Modal.Interface.openModal(
                hardDeleteModal(~id, ~api, ~modal, ~openApplications),
              )}
            variant=Button.Danger>
            {React.string("HARD DELETE")}
          </Button>
        </SessionContext.RequireRole>
      </Button.Panel>
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
        {React.string("This application has ")}
        <Chip.ApplicationStatus value=Success(ApplicationData.Unverified) />
        {React.string(
          " status. This means it was succefully submitted but applicant never clicked on verication link in the email.",
        )}
      </p>
      <p>
        <strong> {React.string("These are the recommended steps to take:")} </strong>
      </p>
      <ol>
        <li>
          {React.string(
            "When was this application submitted? Was email verification sent? Have a look into ",
          )}
          <code> {React.string("Metadata")} </code>
          {React.string(" tab bellow.")}
        </li>
        <li>
          {React.string(
            "If it's very recent, and verification was sent, you should just give them more time to verify the email.",
          )}
        </li>
        <li>
          {React.string("If we miss ")}
          <code> {React.string("Verification Sent")} </code>
          {React.string(" then there is most likely an ")}
          <strong> {React.string("issue on our side")} </strong>
          {React.string(". Try to ")}
          <a onClick=resend> {React.string("re-send the confirmation email")} </a>
          {React.string(" and check that it was sent successfully after a minute or so.")}
        </li>
        <li>
          {React.string("Applications that are already a few days old deserve more attention:")}
          <ol>
            <li>
              {React.string("
                    Review the application to check it's not a spam. If it
                ")}
              <strong> {React.string("looks like a spam then you should report it")} </strong>
              {React.string(" and ")}
              <a onClick=reject> {React.string("invalidate")} </a>
              {React.string(" since it's ilegitimate.")}
            </li>
            <li>
              {React.string("
                    Check for a duplicates.
                    Applicant might had just found a mistake in this application and decided to create a new one.
                    If that's the case you should just
                ")}
              <a onClick=reject> {React.string("invalidate this version of application")} </a>
              {React.string(".")}
            </li>
            <li>
              {React.string("
                    If everything looks legitimate you can
                ")}
              <a onClick=resend> {React.string("re-send the verification email again")} </a>
              {React.string(".")}
            </li>
            <li>
              <strong> {React.string("Get in touch with the applicant")} </strong>
              {React.string(" and try to sort thigs out over the phone. You can also ")}
              <a> {React.string("mark application as verified")} </a>
              {React.string(" manually if necessary.")}
            </li>
            <li>
              {React.string("
                    If nothing worked then there is nothing we can do but to
                ")}
              <a onClick=reject> {React.string("invalidate the application")} </a>
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

  let status = RemoteData.map(detail, ApplicationData.getStatus)

  let openApplications = () => {
    RescriptReactRouter.push(status->RemoteData.toOption->Applications.tabToUrl)
  }

  <Page requireAnyRole=[ListApplications, ViewApplication] mainResource=detail>
    <header className={styles["header"]}>
      <h1 className={styles["title"]}>
        {React.string("Application ")}
        <span className={styles["titleId"]}>
          {switch detail {
          | Success(d) => d.id->Uuid.toString->React.string
          | _ => React.string("...")
          }}
        </span>
      </h1>
      <Page.BackButton
        name="applications" path={status->RemoteData.toOption->Applications.tabToUrl}
      />
      <h2 className={styles["status"]}>
        {React.string("Status:")}
        <Chip.ApplicationStatus value=status />
      </h2>
    </header>
    {viewMessage(
      status,
      ~reject=_ =>
        modal->Modal.Interface.openModal(
          Actions.rejectModal(~id, ~api, ~setDetail, ~modal, ~openApplications),
        ),
      ~resend=_ =>
        modal->Modal.Interface.openModal(Actions.resendModal(~id, ~api, ~modal, ~openApplications)),
    )}
    <div className={styles["personalInfo"]}>
      <DataGrid layout data=detail />
    </div>
    <Tabbed.Tabs>
      <Tabbed.Tab value=Files handlers=tabHandlers> {React.string("Files")} </Tabbed.Tab>
      <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
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
    {switch RemoteData.map(detail, ApplicationData.getStatus) {
    | Success(status) => <Actions id api modal setDetail status openApplications />
    | _ => React.null
    }}
  </Page>
}
