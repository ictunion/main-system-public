@module external styles: {..} = "./MemberDetail/styles.module.scss"

open Data
open Belt

type welcomeEmailPreferences = {
  csSubject: string,
  enSubject: string,
  csTemplate: string,
  enTemplate: string,
  variables: MemberData.detail => Js.Dict.t<string>,
}

module Actions = {
  module Send = {
    @react.component
    let make = (~modal, ~api, ~id, ~body, ~subject, ~variables) => {
      let (error, setError) = React.useState(() => None)
      let sendEmail = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.postJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/send_email",
            ~decoder=Api.Decode.acceptedResponse,
            ~body=MemberData.Encode.emailInfo({body, subject, variables}),
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

  let sendModal = (~modal, ~api, ~id, ~body, ~subject, ~variables): Modal.modalContent => {
    title: "Send email",
    content: <Send modal api id body subject variables />,
  }

  @react.component
  let make = (~status, ~modal, ~api, ~id, ~body, ~subject, ~variables) => {
    switch status {
    | MemberData.NewMember =>
      <Button.Panel>
        <Button
          variant=Button.Cta
          onClick={_ =>
            modal->Modal.Interface.openModal(
              sendModal(~modal, ~api, ~id, ~body, ~subject, ~variables),
            )}>
          {React.string("Send")}
        </Button>
      </Button.Panel>
    | MemberData.CurrentMember => React.null
    | MemberData.PastMember => React.null
    }
  }
}

@react.component
let make = (~api, ~id, ~modal, ~preferences: welcomeEmailPreferences) => {
  let backPath = "/members/" ++ Uuid.toString(id)

  let (detail: Api.webData<MemberData.detail>, setDetail) = React.useState(RemoteData.init)
  let (text, setText) = React.useState(_ => "")
  let onChange = evt => {
    let emailTxt = ReactEvent.Form.target(evt)["value"]
    setText(_prev => emailTxt)
  }
  let (subject, setSubject) = React.useState(_ => "")
  let (variables, setVariables) = React.useState(_ => Js.Dict.empty())

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
      setText(
        _ =>
          if memberLanguage == "cs" {
            preferences.csTemplate
          } else {
            preferences.enTemplate
          },
      )
      setSubject(
        _ =>
          if memberLanguage == "cs" {
            preferences.csSubject
          } else {
            preferences.enSubject
          },
      )
      setVariables(_ => RemoteData.unwrap(detail, ~default=Js.Dict.empty(), preferences.variables))
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
    <textarea onChange name="body" value=text rows=40 />
    {switch status {
    | Success(s) => <Actions status=s modal api id body=text subject variables />
    | _ => React.null
    }}
  </Page>
}
