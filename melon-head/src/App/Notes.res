@module external styles: {..} = "./Notes/styles.module.scss"

open Data

@val external confirm: string => bool = "confirm"
@get external key: Dom.event => string = "key"
@val @scope("window")
external addWindowListener: (string, Dom.event => unit) => unit = "addEventListener"
@val @scope("window")
external removeWindowListener: (string, Dom.event => unit) => unit = "removeEventListener"

let pathOf = (target: NoteData.target) => "/notes/" ++ NoteData.targetToString(target)

/* Latest note and count of every target of one type that has notes, keyed by
   target id. One request per table, not per row; the full thread is only
   loaded when the drawer opens. Call `refresh` after a change in the drawer. */
let useSummary = (api: Api.t, target: NoteData.target) => {
  let (data, _, send) =
    api->Hook.getData(~path=pathOf(target), ~decoder=Json.Decode.array(NoteData.Decode.summary))

  let byId = React.useMemo1(() =>
    data->RemoteData.unwrap(~default=Map.make(), rows =>
      rows
      ->Array.map((s: NoteData.summary) => (Uuid.toString(s.targetId), s))
      ->Map.fromArray
    )
  , [data])

  let refresh = () => {
    let _ = send()
  }

  (byId, data, refresh)
}

let summaryOf = (byId: Map.t<string, NoteData.summary>, id: Uuid.t) =>
  byId->Map.get(Uuid.toString(id))

/* Table cell: latest note and the total count, e.g. "called on Monday [3]".
 No author or date, to keep the column narrow. */
module Cell = {
  @react.component
  let make = (~summary: option<NoteData.summary>, ~onOpen: unit => unit) =>
    <button type_="button" className={styles["cell"]} onClick={_ => onOpen()}>
      {switch summary {
      | Some(s) =>
        <>
          <span className={styles["cellText"]}> {React.string(s.latestBody)} </span>
          <span className={styles["cellCount"]}>
            {React.string("[" ++ Int.toString(s.count) ++ "]")}
          </span>
        </>
      | None => <span className={styles["cellEmpty"]}> {React.string("Add note")} </span>
      }}
    </button>
}

module Editor = {
  @react.component
  let make = (~initial="", ~submitLabel: string, ~onSubmit: string => unit, ~onCancel=?) => {
    let (text, setText) = React.useState(() => initial)
    let trimmed = String.trim(text)

    <form
      className={styles["editor"]}
      onSubmit={e => {
        ReactEvent.Form.preventDefault(e)
        if trimmed != "" {
          onSubmit(trimmed)
        }
      }}
    >
      <textarea
        className={styles["textarea"]}
        rows=3
        value=text
        onChange={e => {
          // Read before setText: React runs the updater after the event, when currentTarget is null.
          let value = ReactEvent.Form.currentTarget(e)["value"]
          setText(_ => value)
        }}
      />
      <Button.Panel>
        {switch onCancel {
        | Some(cancel) => <Button onClick={_ => cancel()}> {React.string("Cancel")} </Button>
        | None => React.null
        }}
        <Button type_="submit" variant=Button.Cta disabled={trimmed == ""}>
          {React.string(submitLabel)}
        </Button>
      </Button.Panel>
    </form>
  }
}

/* The whole thread of one target: add, edit and delete. Edit and Delete show
   for the author and for `manage-notes` holders; the server enforces the same
   rule. Remount it (via `key`) to switch targets -- it only fetches on mount. */
module Panel = {
  @react.component
  let make = (
    ~api: Api.t,
    ~target: NoteData.target,
    ~targetId: Uuid.t,
    ~onChange: unit => unit,
  ) => {
    let session = React.useContext(SessionContext.context)
    let path = pathOf(target) ++ "/" ++ Uuid.toString(targetId)

    let (notes, _, refresh) =
      api->Hook.getData(~path, ~decoder=Json.Decode.array(NoteData.Decode.note))
    let (editing, setEditing) = React.useState(() => None)
    // Bumped after each successful change to remount (and so empty) the add form.
    let (editorKey, setEditorKey) = React.useState(() => 0)
    let (error, setError) = React.useState(() => None)

    let isManager = switch session {
    | Success(s) => s->Session.hasRole(~role=ManageNotes)
    | _ => false
    }

    let canModify = (n: NoteData.note) =>
      isManager ||
      switch (session, n.authorSub) {
      | (Success(s), Some(sub)) => sub == s.tokenClaims.sub
      | _ => false
      }

    let run = req =>
      req->Future.get(res =>
        switch res {
        | Ok(_) => {
            setError(_ => None)
            setEditing(_ => None)
            setEditorKey(k => k + 1)
            let _ = refresh()
            onChange()
          }
        | Error(e) => setError(_ => Some(e))
        }
      )

    let add = body =>
      api
      ->Api.postJson(~path, ~decoder=NoteData.Decode.note, ~body=NoteData.Encode.body(body))
      ->run

    let update = (id, body) =>
      api
      ->Api.patchJson(
        ~path=path ++ "/" ++ Uuid.toString(id),
        ~decoder=NoteData.Decode.note,
        ~body=NoteData.Encode.body(body),
      )
      ->run

    let remove = id =>
      if confirm("This permanently deletes the note.") {
        api
        ->Api.deleteJson(
          ~path=path ++ "/" ++ Uuid.toString(id),
          ~decoder=Api.Decode.acceptedResponse,
          ~body=None,
        )
        ->run
      }

    let clearAll = count =>
      if confirm("This permanently deletes all " ++ Int.toString(count) ++ " notes.") {
        api->Api.deleteJson(~path, ~decoder=Api.Decode.acceptedResponse, ~body=None)->run
      }

    <div className={styles["panel"]}>
      {switch target {
      | MemberDues =>
        <p className={styles["hint"]}>
          {React.string("Workplace reps of this member can read and reply to these notes.")}
        </p>
      | _ => React.null
      }}
      <Editor key={Int.toString(editorKey)} submitLabel="Add note" onSubmit=add />
      {switch error {
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | None => React.null
      }}
      {switch notes {
      | Success([]) => <p className={styles["hint"]}> {React.string("No notes yet.")} </p>
      | Success(rows) =>
        <>
          <ul className={styles["list"]}>
            {rows
            ->Array.map(n =>
              <li key={Uuid.toString(n.id)} className={styles["note"]}>
                <header className={styles["noteHeader"]}>
                  <strong> {React.string(n.authorName)} </strong>
                  <span> {React.string(n.createdAt->Date.toLocaleString)} </span>
                </header>
                {switch editing {
                | Some(id) if id == n.id =>
                  <Editor
                    initial=n.body
                    submitLabel="Save"
                    onSubmit={body => update(n.id, body)}
                    onCancel={() => setEditing(_ => None)}
                  />
                | _ =>
                  <>
                    <p className={styles["body"]}> {React.string(n.body)} </p>
                    {canModify(n)
                      ? <div className={styles["actions"]}>
                          <a onClick={_ => setEditing(_ => Some(n.id))}> {React.string("Edit")} </a>
                          <a onClick={_ => remove(n.id)}> {React.string("Delete")} </a>
                        </div>
                      : React.null}
                  </>
                }}
              </li>
            )
            ->React.array}
          </ul>
          {isManager
            ? <Button.Panel>
                <Button variant=Button.Danger onClick={_ => clearAll(Array.length(rows))}>
                  {React.string("Clear all notes")}
                </Button>
              </Button.Panel>
            : React.null}
        </>
      | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | Idle
      | Loading =>
        <div className={styles["loading"]}>
          <Icons.Loading variant=Icons.Dark />
        </div>
      }}
    </div>
  }
}

/* Right-side drawer, so the table row stays visible while working through a
 list. Closes on Escape, a click on the backdrop or the close button. */
module Drawer = {
  @react.component
  let make = (~title: React.element, ~onClose: unit => unit, ~children) => {
    React.useEffect0(() => {
      let onKeyDown = e =>
        if e->key == "Escape" {
          onClose()
        }
      addWindowListener("keydown", onKeyDown)
      Some(() => removeWindowListener("keydown", onKeyDown))
    })

    <>
      <div className={styles["backdrop"]} onClick={_ => onClose()} />
      <aside className={styles["drawer"]} role="dialog" ariaLabel="Notes">
        <header className={styles["drawerHeader"]}>
          <span> title </span>
          <button className={styles["closeBtn"]} title="Close" onClick={_ => onClose()}>
            <Icons.Close />
          </button>
        </header>
        <div className={styles["drawerBody"]}> children </div>
      </aside>
    </>
  }
}
