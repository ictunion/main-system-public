module Editor = {
  @react.component
  let make = (
    ~modal: Modal.Interface.t,
    ~saveNote: string => Future.t<result<'a, Api.error>>,
    ~onSaved: unit => unit,
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
      saveNote(note)->Future.get(res => {
        switch res {
        | Ok(_data) => {
            onSaved()
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
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
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
}

let modalContent = (~modal, ~saveNote, ~refresh, initialNote): Modal.modalContent => {
  let onSaved = () => {
    let _ = refresh()
  }

  {
    title: "Update note",
    content: <Editor modal saveNote onSaved initialNote />,
  }
}

module Member = {
  let modal = (~api: Api.t, ~modal, ~refresh, uuid, initialNote) => {
    let saveNote = note =>
      api->Api.patchJson(
        ~path="/members/" ++ Data.Uuid.toString(uuid) ++ "/note",
        ~decoder=MemberData.Decode.detail,
        ~body=NoteData.Encode.note(note),
      )
    modalContent(~modal, ~saveNote, ~refresh, initialNote)
  }
}

module Application = {
  let modal = (~api: Api.t, ~modal, ~refresh, uuid, initialNote) => {
    let saveNote = note =>
      api->Api.patchJson(
        ~path="/applications/" ++ Data.Uuid.toString(uuid) ++ "/note",
        ~decoder=ApplicationData.Decode.detail,
        ~body=NoteData.Encode.note(note),
      )
    modalContent(~modal, ~saveNote, ~refresh, initialNote)
  }
}
