@module external styles: {..} = "./MemberDetail/styles.module.scss"

open Data
open Belt

module Actions = {
  open MemberData

  module Send = {
    type acceptTabs =
      | Create
      | Pair

    @react.component
    let make = (~modal, ~api, ~id, ~template) => {
      let (error, setError) = React.useState(() => None)
      let sendEmail = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.postJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/workplace_welcome_email",
            ~decoder=Api.Decode.acceptedResponse,
            ~body=MemberData.Encode.newEmailInfo(template),
          )

        req->Future.get(res => {
          switch res {
          | Ok(_) => Modal.Interface.closeModal(modal)
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <div className={styles["modalBody"]}>
          <p> {React.string("Send email to the new member")} </p>
          {switch error {
          | None => React.null
          | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
          }}
        </div>
        <Button.Panel>
          <Button onClick={_ => modal->Modal.Interface.closeModal}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Cta onClick=sendEmail> {React.string("Send email")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  let sendModal = (~modal, ~api, ~id, ~template): Modal.modalContent => {
    title: "Send email",
    content: <Send modal api id template />,
  }

  @react.component
  let make = (~status, ~modal, ~api, ~id, ~template) => {
    switch status {
    | NewMember =>
      <Button.Panel>
        <Button
          variant=Button.Cta
          onClick={_ => modal->Modal.Interface.openModal(sendModal(~modal, ~api, ~id, ~template))}>
          {React.string("Send")}
        </Button>
      </Button.Panel>
    | CurrentMember => React.null
    | PastMember => React.null
    }
  }
}

@react.component
let make = (~api, ~id, ~modal) => {
  let en: string = %raw("require('../../raw/welcome_workplace_email_en.mjml').default")
  let cs: string = %raw("require('../../raw/welcome_workplace_email_cs.mjml').default")

  // back url
  let backPath = "/members/" ++ Uuid.toString(id)

  let (detail: Api.webData<MemberData.detail>, setDetail) = React.useState(RemoteData.init)
  let (text, setText) = React.useState(_ => "")
  let onChange = evt => {
    let emailTxt = ReactEvent.Form.target(evt)["value"]
    setText(_prev => emailTxt)
  }

  React.useEffect0(() => {
    let req =
      api->Api.getJson(~path="/members/" ++ Uuid.toString(id), ~decoder=MemberData.Decode.detail)
    setDetail(RemoteData.setLoading)

    req->Future.get(res => {
      let detail = RemoteData.fromResult(res)
      setDetail(_ => detail)
      let memberLanguage: string = RemoteData.unwrap(
        detail,
        ~default="cannot happen",
        member => member.language->Option.getWithDefault("en"),
      )
      let template = if memberLanguage == "cs" {
        cs
      } else {
        en
      }
      setText(_prev => template)
    })

    Some(() => Future.cancel(req))
  })

  let status = RemoteData.map(detail, MemberData.getStatus)

  <Page requireAnyRole=[ListMembers, ViewMember] mainResource=detail>
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
      <Page.BackButton name="members" path=backPath />
    </header>
    // editable MJML template here
    <textarea onChange name="template" value=text rows=40 />
    {switch status {
    | Success(s) => <Actions status=s modal api id template=text />
    | _ => React.null
    }}
  </Page>
}
