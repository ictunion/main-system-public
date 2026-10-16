@module external styles: {..} = "./Board/styles.module.scss"

open Data
open Belt

type tab = Current | Past

/* react-datepicker calls onChange with null when the date is cleared, which
   the Form.DatePicker binding doesn't model. */
external toNullable: Js.Date.t => Js.Nullable.t<Js.Date.t> = "%identity"

let memberLabel = (m: MemberData.summary): string => {
  let name = [m.firstName, m.lastName]->Array.keepMap(x => x)->Js.Array2.joinWith(" ")
  "#" ++ Int.toString(m.memberNumber) ++ " " ++ name
}

let viewError = error =>
  switch error {
  | None => React.null
  | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
  }

/* Picks a current member by typing a name or number (matched by the browser
   against the datalist labels), or by pasting the member ID directly. */
module MemberPicker = {
  @react.component
  let make = (
    ~members: Api.webData<array<MemberData.summary>>,
    ~activeOffices: array<OfficeData.office>,
    ~value: string,
    ~onChange: string => unit,
  ) => {
    let onInput = event => onChange(ReactEvent.Form.currentTarget(event)["value"])
    let trimmed = Js.String2.trim(value)

    <Form.Field>
      <Form.Label>
        {React.string("Member (name, number or ID)")}
        <input list="board-members" placeholder="Jane Doe" value onInput />
      </Form.Label>
      {switch members {
      | Success(members) =>
        <>
          <datalist id="board-members">
            {members
            ->Array.map(m =>
              <option key={Uuid.toString(m.id)} value={Uuid.toString(m.id)}>
                {React.string(memberLabel(m))}
              </option>
            )
            ->React.array}
          </datalist>
          {if trimmed == "" {
            React.null
          } else {
            switch members->Js.Array2.find(m => Uuid.toString(m.id) == trimmed) {
            | None =>
              <Message.Warning>
                {React.string("No current member with this ID. Pick one from the list.")}
              </Message.Warning>
            | Some(m) =>
              <>
                <Message.Info> {React.string("Selected: " ++ memberLabel(m))} </Message.Info>
                {switch activeOffices->Js.Array2.find(o => o.memberId == m.id) {
                | None => React.null
                | Some(o) =>
                  <Message.Warning>
                    {React.string(
                      "This member currently holds the office \"" ++
                      o.title ++ "\". End it first if the new office replaces it.",
                    )}
                  </Message.Warning>
                }}
              </>
            }
          }}
        </>
      | Failure(err) => viewError(Some(err))
      | _ => React.null
      }}
    </Form.Field>
  }
}

/* Create a new office (`existing = None`) or edit an active one. */
module Editor = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~existing: option<OfficeData.office>,
    ~activeOffices: array<OfficeData.office>,
    ~onSaved: unit => unit,
  ) => {
    let (memberId, setMemberId) = React.useState(_ => "")
    let (title, setTitle) = React.useState(_ => existing->Option.mapWithDefault("", o => o.title))
    let (electedBy, setElectedBy) = React.useState(_ =>
      existing->Option.mapWithDefault("", o => o.electedBy)
    )
    let (electedOn, setElectedOn) = React.useState(_ => existing->Option.map(o => o.electedOn))
    let (termEndsOn, setTermEndsOn) = React.useState(_ =>
      existing->Option.flatMap(o => o.termEndsOn)
    )
    // Once the admin sets the term end by hand, picking the election date no
    // longer overwrites it.
    let (termEndTouched, setTermEndTouched) = React.useState(_ => false)

    let onElectedOnChange = d => {
      let date = d->toNullable->Js.Nullable.toOption
      setElectedOn(_ => date)

      // New offices only: prefill the usual three-year term
      if Option.isNone(existing) && !termEndTouched {
        setTermEndsOn(_ => date->Option.map(OfficeData.addYears(_, 3)))
      }
    }

    let onTermEndsOnChange = d => {
      setTermEndTouched(_ => true)
      setTermEndsOn(_ => d->toNullable->Js.Nullable.toOption)
    }
    let (error, setError) = React.useState(() => None)

    let (members, _, _) =
      api->Hook.getData(
        ~enabled=Option.isNone(existing),
        ~path="/members/current",
        ~decoder=Json.Decode.array(MemberData.Decode.summary),
      )

    let memberKnown = switch (existing, members) {
    | (Some(_), _) => true
    | (None, Success(members)) =>
      members->Array.some(m => Uuid.toString(m.id) == Js.String2.trim(memberId))
    // Members failed to load: let the API validate a pasted ID
    | (None, _) => Js.String2.trim(memberId) != ""
    }

    let isFilled = s => Js.String2.trim(s) != ""

    let canSubmit = memberKnown && isFilled(title) && isFilled(electedBy) && Option.isSome(electedOn)

    let onSubmit = _ => {
      switch electedOn {
      | None => ()
      | Some(electedOn) => {
          let req = switch existing {
          | None =>
            api->Api.postJson(
              ~path="/members/offices",
              ~decoder=OfficeData.Decode.office,
              ~body=OfficeData.Encode.newOffice(
                ~memberId=Js.String2.trim(memberId),
                ~title,
                ~electedBy,
                ~electedOn,
                ~termEndsOn,
              ),
            )
          | Some(office) =>
            api->Api.patchJson(
              ~path="/members/offices/" ++ Uuid.toString(office.id),
              ~decoder=OfficeData.Decode.office,
              ~body=OfficeData.Encode.patch(~title, ~electedBy, ~electedOn, ~termEndsOn),
            )
          }

          req->Future.get(res => {
            switch res {
            | Ok(_) => {
                onSaved()
                Modal.Interface.closeModal(modal)
              }
            | Error(e) => setError(_ => Some(e))
            }
          })
        }
      }
    }

    <Form onSubmit>
      {switch existing {
      | None => <MemberPicker members activeOffices value=memberId onChange={v => setMemberId(_ => v)} />
      | Some(office) =>
        <Form.Field>
          <Form.Label> {React.string("Member")} </Form.Label>
          <p>
            {MemberSummaryTable.viewPaddedNumber(office.memberNumber, ())}
            {React.string(" " ++ OfficeData.fullName(office))}
          </p>
        </Form.Field>
      }}
      <Form.SelectField
        label="Office"
        placeholder="Select office"
        options=OfficeData.titles
        value=title
        onChange={v => setTitle(_ => v)}
      />
      <Form.SelectField
        label="Elected by"
        placeholder="Select who elected the member"
        options=OfficeData.electedByOptions
        value=electedBy
        onChange={v => setElectedBy(_ => v)}
      />
      <Form.Date
        label="Elected on"
        value=electedOn
        onChange=onElectedOnChange
      />
      <Form.Date
        label="Term ends on (optional, last day in office)"
        value=termEndsOn
        onChange=onTermEndsOnChange
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta disabled={!canSubmit}>
          {React.string(Option.isSome(existing) ? "Save changes" : "Assign office")}
        </Button>
      </Button.Panel>
      {viewError(error)}
    </Form>
  }
}

module Confirm = {
  @react.component
  let make = (
    ~modal: Modal.Interface.t,
    ~message: string,
    ~confirmLabel: string,
    ~request: unit => Future.t<result<Api.acceptedResponse, Api.error>>,
    ~onDone: unit => unit,
  ) => {
    let (error, setError) = React.useState(() => None)

    let onConfirm = _ =>
      request()->Future.get(res => {
        switch res {
        | Ok(_) => {
            onDone()
            Modal.Interface.closeModal(modal)
          }
        | Error(e) => setError(_ => Some(e))
        }
      })

    <Modal.Content>
      <p> {React.string(message)} </p>
      <Button.Panel>
        <Button onClick={_ => modal->Modal.Interface.closeModal}> {React.string("Cancel")} </Button>
        <Button variant=Button.Danger onClick=onConfirm> {React.string(confirmLabel)} </Button>
      </Button.Panel>
      {viewError(error)}
    </Modal.Content>
  }
}

/* Soft delete of a record entered by mistake; it disappears from both tabs. */
let deleteOffice = (~api: Api.t, office: OfficeData.office) =>
  api->Api.deleteJson(
    ~path="/members/offices/" ++ Uuid.toString(office.id),
    ~decoder=Api.Decode.acceptedResponse,
    ~body=None,
  )

let viewDay = (date: Js.Date.t) => date->Js.Date.toLocaleDateString->React.string

/* Ends a current office early (e.g. resignation) by setting the last day in
   office. It defaults to today, as the update may be entered a few days
   later. Offices reaching their planned term end move to Past on their
   own. */
module EndOffice = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~office: OfficeData.office,
    ~onDone: unit => unit,
  ) => {
    let (termEndsOn, setTermEndsOn) = React.useState(_ => Some(OfficeData.today()))
    let (error, setError) = React.useState(() => None)

    let invalid = switch termEndsOn {
    | None => Some("Pick the last day in office.")
    | Some(date) if Js.Date.getTime(date) > Js.Date.getTime(OfficeData.today()) =>
      Some("The last day in office can't be in the future.")
    | Some(date) if Js.Date.getTime(date) < Js.Date.getTime(office.electedOn) =>
      Some(
        "The last day in office can't be before the election (" ++
        Js.Date.toLocaleDateString(office.electedOn) ++ ").",
      )
    | Some(_) => None
    }

    let onSubmit = _ =>
      api
      ->Api.patchJson(
        ~path="/members/offices/" ++ Uuid.toString(office.id),
        ~decoder=OfficeData.Decode.office,
        ~body=OfficeData.Encode.termEnd(termEndsOn),
      )
      ->Future.get(res => {
        switch res {
        | Ok(_) => {
            onDone()
            Modal.Interface.closeModal(modal)
          }
        | Error(e) => setError(_ => Some(e))
        }
      })

    <Form onSubmit>
      <p>
        {React.string(
          "End the office \"" ++
          office.title ++
          "\" held by " ++
          OfficeData.fullName(office) ++ ". It stays current until the end of the last day in office, then moves to Past.",
        )}
      </p>
      <Form.Date
        label="Last day in office"
        value=termEndsOn
        onChange={d => setTermEndsOn(_ => d->toNullable->Js.Nullable.toOption)}
      />
      {switch invalid {
      | None => React.null
      | Some(msg) => <Message.Warning> {React.string(msg)} </Message.Warning>
      }}
      <Button.Panel>
        <Button type_="button" onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Careful disabled={Option.isSome(invalid)}>
          {React.string("End office")}
        </Button>
      </Button.Panel>
      {viewError(error)}
    </Form>
  }
}

let columns = (
  ~api: Api.t,
  ~modal: Modal.Interface.t,
  ~activeOffices: array<OfficeData.office>,
  ~refresh: unit => unit,
  ~tab: tab,
): array<DataTable.column<OfficeData.office>> => {
  let openEdit = (office: OfficeData.office) =>
    modal->Modal.Interface.openModal({
      title: "Edit office",
      content: <Editor api modal existing=Some(office) activeOffices onSaved=refresh />,
    })

  let openDelete = (~message, office) =>
    modal->Modal.Interface.openModal({
      title: "Delete record",
      content: <Confirm
        modal
        message
        confirmLabel="Delete record"
        request={() => deleteOffice(~api, office)}
        onDone=refresh
      />,
    })

  let openEnd = (office: OfficeData.office) =>
    modal->Modal.Interface.openModal({
      title: "End office",
      content: <EndOffice api modal office onDone=refresh />,
    })

  let describe = (office: OfficeData.office) =>
    "\"" ++ office.title ++ "\" held by " ++ OfficeData.fullName(office)

  let actions = (office: OfficeData.office) =>
    <SessionContext.RequireRole anyOf=[Session.ManageMembers]>
      <div className={styles["actions"]}>
        // Past offices are read-only; they can only be deleted as mistakes
        {switch tab {
        | Current =>
          <>
            <Button onClick={_ => openEdit(office)}> {React.string("Edit")} </Button>
            <Button variant=Button.Careful onClick={_ => openEnd(office)}>
              {React.string("End")}
            </Button>
          </>
        | Past => React.null
        }}
        <Button
          variant=Button.Danger
          onClick={_ =>
            openDelete(
              ~message="Delete the record of the office " ++
              describe(office) ++ "? It will be hidden everywhere, including Past. Use this only for records entered by mistake; to end an office, use End.",
              office,
            )}>
          {React.string("Delete")}
        </Button>
      </div>
    </SessionContext.RequireRole>

  let base: array<DataTable.column<OfficeData.office>> = [
    {
      name: "Number",
      minMax: ("100px", "1fr"),
      view: o => MemberSummaryTable.viewPaddedNumber(o.memberNumber, ()),
    },
    {
      name: "Name",
      minMax: ("150px", "2fr"),
      view: o =>
        <a onClick={_ => RescriptReactRouter.push("/members/" ++ Uuid.toString(o.memberId))}>
          {React.string(OfficeData.fullName(o))}
        </a>,
    },
    {name: "Office", minMax: ("150px", "2fr"), view: o => React.string(o.title)},
    {name: "Elected by", minMax: ("150px", "2fr"), view: o => React.string(o.electedBy)},
    {name: "Elected on", minMax: ("110px", "1fr"), view: o => viewDay(o.electedOn)},
    {
      name: "Term ends",
      minMax: ("110px", "1fr"),
      view: o =>
        o.termEndsOn->View.option(viewDay),
    },
  ]

  let actionsColumn: DataTable.column<OfficeData.office> = {
    name: "",
    minMax: ("260px", "260px"),
    view: actions,
  }

  Array.concat(base, [actionsColumn])
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  let tabHandlers = Tabbed.make(Current)
  let (activeTab, _) = tabHandlers

  let decoder = Json.Decode.array(OfficeData.Decode.office)
  let (current, _, sendCurrent) = api->Hook.getData(~path="/members/offices", ~decoder)
  let (past, _, sendPast) = api->Hook.getData(~path="/members/offices/past", ~decoder)

  let refresh = () => {
    let _ = sendCurrent()
    let _ = sendPast()
  }

  let activeOffices = switch current {
  | Success(offices) => offices
  | _ => []
  }

  let openAssign = _ =>
    modal->Modal.Interface.openModal({
      title: "Assign office",
      content: <Editor api modal existing=None activeOffices onSaved=refresh />,
    })

  let columns = columns(~api, ~modal, ~activeOffices, ~refresh, ~tab=activeTab)

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Board")} </Page.Title>
    <SessionContext.RequireRole anyOf=[Session.ManageMembers]>
      <Button.Panel>
        <Button onClick=openAssign> {React.string("Assign office")} </Button>
      </Button.Panel>
    </SessionContext.RequireRole>
    <div className={styles["mainContent"]}>
      <Tabbed.Tabs>
        <Tabbed.Tab value=Current handlers=tabHandlers> {React.string("Current")} </Tabbed.Tab>
        <Tabbed.Tab value=Past handlers=tabHandlers> {React.string("Past")} </Tabbed.Tab>
      </Tabbed.Tabs>
      <Tabbed.Content tab=Current handlers=tabHandlers>
        <DataTable data=current columns>
          <p> {React.string("Nobody holds an office.")} </p>
        </DataTable>
      </Tabbed.Content>
      <Tabbed.Content tab=Past handlers=tabHandlers>
        <DataTable data=past columns>
          <p> {React.string("There are no past offices.")} </p>
        </DataTable>
      </Tabbed.Content>
    </div>
  </Page>
}
