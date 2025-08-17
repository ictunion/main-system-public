open Data

@react.component
let make = (
  ~api: Api.t,
  ~modal: Modal.Interface.t,
  ~refreshMembers,
  ~uuid: Uuid.t,
  ~initialNote: option<string>,
) => {
  let (note, setNote) = React.useState(_ =>
    switch initialNote {
    | None => ""
    | Some(v) => v
    }
  )
  let (error, setError) = React.useState(() => None)

  let onSubmit = _ => {
    let body: Js.Json.t = ApplicationData.Encode.newNote(note)
    let path = "/members/" ++ Uuid.toString(uuid) ++ "/note"
    let req = api->Api.patchJson(~path, ~decoder=MemberData.Decode.detail, ~body)

    req->Future.get(res => {
      switch res {
      | Ok(_data) => {
          let _ = refreshMembers()
          Modal.Interface.closeModal(modal)
        }
      | Error(e) => setError(_ => Some(e))
      }
    })
  }

  <Form onSubmit>
    <Form.TextField
      label="Note"
      placeholder="...some text..."
      value=note
      onInput={updated => setNote(_ => updated)}
    />
    <Button.Panel>
      <Button type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
        {React.string("Cancel")}
      </Button>
      <Button type_="submit" variant=Button.Cta> {React.string("Update note")} </Button>
    </Button.Panel>
    {switch error {
    | None => React.null
    | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
    }}
  </Form>
}
