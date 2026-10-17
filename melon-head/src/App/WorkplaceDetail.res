@module external styles: {..} = "./WorkplaceDetail/styles.module.scss"

open Data

let layout: DataGrid.t<WorkplaceData.summary> = [
  {
    label: "Details",
    cells: [
      {
        label: "Name",
        view: d => d.name->React.string,
        minmax: ("200px", "900px"),
      },
      {
        label: "Short Name",
        view: d => d.workplaceShortName->View.option(React.string),
        minmax: ("150px", "300px"),
      },
      {
        label: "State",
        view: d =>
          switch WorkplaceData.getStatus(d) {
          | Initial => "Initial"
          | Established => "Established"
          | Announced => "Announced"
          | Cancelled => "Cancelled"
          }->React.string,
        minmax: ("120px", "200px"),
      },
      {
        label: "Email",
        view: d => <Link.Email email={d.email} />,
        minmax: ("300px", "900px"),
      },
      {
        label: "Members",
        view: d => d.memberCount->Belt.Int.toString->React.string,
        minmax: ("100px", "200px"),
      },
    ],
  },
  {
    label: "Company",
    cells: [
      {
        label: "Company ID (IČ)",
        view: d => d.companyId->View.option(React.string),
        minmax: ("150px", "300px"),
      },
      {
        label: "Company Address",
        view: d => d.companyAddress->View.option(React.string),
        minmax: ("300px", "900px"),
      },
    ],
  },
  {
    label: "Keycloak",
    cells: [
      {
        label: "Group ID",
        view: d => d.keycloakGroupId->Uuid.toString->React.string,
        minmax: ("300px", "900px"),
      },
      {
        label: "Executive Group ID",
        view: d =>
          d.keycloakExecutiveGroupId->Belt.Option.mapWithDefault(React.string("—"), uuid =>
            uuid->Uuid.toString->React.string
          ),
        minmax: ("300px", "900px"),
      },
    ],
  },
  {
    label: "Newsletter",
    cells: [
      {
        label: "Listmonk ID",
        view: d =>
          switch d.newsletterId {
          | None => React.string("Newsletter not connected")
          | Some(id) => id->Belt.Int.toString->React.string
          },
        minmax: ("250px", "600px"),
      },
    ],
  },
  {
    label: "Metadata",
    cells: [
      {
        label: "Created On",
        view: d => d.createdAt->Js.Date.toLocaleDateString->React.string,
        minmax: ("150px", "300px"),
      },
      {
        label: "Established On",
        view: d =>
          d.establishedAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Announced On",
        view: d =>
          d.announcedAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Cancelled On",
        view: d =>
          d.cancelledAt->Belt.Option.mapWithDefault(React.string("—"), date =>
            date->Js.Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
    ],
  },
]

module EditWorkplace = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~id: Uuid.t,
    ~detail: WorkplaceData.summary,
    ~setDetail,
  ) => {
    let (newsletterId, setNewsletterId) = React.useState(_ =>
      detail.newsletterId->Belt.Option.mapWithDefault("", Belt.Int.toString)
    )
    let (shortName, setShortName) = React.useState(_ =>
      detail.workplaceShortName->Belt.Option.getWithDefault("")
    )
    let (companyId, setCompanyId) = React.useState(_ =>
      detail.companyId->Belt.Option.getWithDefault("")
    )
    let (companyAddress, setCompanyAddress) = React.useState(_ =>
      detail.companyAddress->Belt.Option.getWithDefault("")
    )
    let (error, setError) = React.useState(() => None)

    /* Orca keeps the stored value when a field is sent empty, so a value once set
       can be changed but not removed. Say so instead of silently keeping it. */
    let clearedField = {
      let cleared = (isStored: bool, input: string) => isStored && input->Js.String2.trim == ""
      open Belt.Option
      if cleared(detail.newsletterId->isSome, newsletterId) {
        Some("Newsletter ID")
      } else if cleared(detail.workplaceShortName->isSome, shortName) {
        Some("Short Name")
      } else if cleared(detail.companyId->isSome, companyId) {
        Some("Company ID")
      } else if cleared(detail.companyAddress->isSome, companyAddress) {
        Some("Company Address")
      } else {
        None
      }
    }

    let text = (value: string) => {
      let trimmed = value->Js.String2.trim
      trimmed == "" ? Json.Encode.null : Json.Encode.string(trimmed)
    }

    let onSubmit = _ => {
      let parsedId = switch newsletterId->Js.String2.trim {
      | "" => Ok(None)
      | s =>
        switch Belt.Int.fromString(s) {
        | Some(n) => Ok(Some(n))
        | None => Error("Newsletter ID must be a number")
        }
      }
      switch (parsedId, clearedField) {
      | (Error(msg), _) => setError(_ => Some(msg))
      | (_, Some(field)) => setError(_ => Some(field ++ " can be changed, but not removed"))
      | (Ok(parsedNewsletterId), None) =>
        let body = Json.Encode.object([
          ("newsletter_id", Json.Encode.option(Json.Encode.int, parsedNewsletterId)),
          ("workplace_short_name", text(shortName)),
          ("company_id", text(companyId)),
          ("company_address", text(companyAddress)),
        ])
        let req =
          api->Api.patchJson(
            ~path="/workplaces/" ++ Uuid.toString(id),
            ~decoder=WorkplaceData.Decode.summary,
            ~body,
          )
        req->Future.get(res => {
          switch res {
          | Ok(updated) => {
              setDetail(_ => RemoteData.Success(updated))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e->Api.showError))
          }
        })
      }
    }

    <Form onSubmit>
      <Form.TextField
        label="Newsletter ID"
        placeholder="42"
        value=newsletterId
        onInput={v => setNewsletterId(_ => v)}
      />
      <Form.TextField
        label="Short Name"
        placeholder="ACME CZ"
        value=shortName
        onInput={v => setShortName(_ => v)}
      />
      <Form.TextField
        label="Company ID (IČ)"
        placeholder="12345678"
        value=companyId
        onInput={v => setCompanyId(_ => v)}
      />
      <Form.TextField
        label="Company Address"
        placeholder="Vinohradská 1234/56, 120 00 Praha 2"
        value=companyAddress
        onInput={v => setCompanyAddress(_ => v)}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Save")} </Button>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(msg) => <Message.Error> {React.string(msg)} </Message.Error>
      }}
    </Form>
  }
}

let editWorkplaceModal = (~api, ~modal, ~id, ~detail, ~setDetail): Modal.modalContent => {
  title: "Edit Workplace",
  content: <EditWorkplace api modal id detail setDetail />,
}

/* Documents Orca can generate for a workplace. `value` is the `document_type`
   the API expects. */
type documentType = {
  value: string,
  label: string,
}

let documentTypes = [
  {value: "entrustment", label: "Entrustment (Pověření)"},
  {
    value: "workplace_announcement",
    label: "Notification and Proof of Trade Union Operation (Oznámení a prokázání působnosti odborové organizace)",
  },
  {value: "member_confirmation", label: "Confirmation of Membership (Potvrzení o členství)"},
]

/* `YYYY-MM-DD` from the local calendar day. The date picker returns local
   midnight, which `toJSON` (UTC) would shift to the previous day in Prague. */
let localDay = (date: Js.Date.t): string => {
  let pad = n => (n < 10 ? "0" : "") ++ Belt.Int.toString(n)
  Belt.Int.toString(Belt.Float.toInt(Js.Date.getFullYear(date))) ++
  "-" ++
  pad(Belt.Float.toInt(Js.Date.getMonth(date)) + 1) ++
  "-" ++
  pad(Belt.Float.toInt(Js.Date.getDate(date)))
}

let isUuid = (s: string): bool =>
  %re("/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i")->Js.Re.test_(s)

module GenerateDocument = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~detail: WorkplaceData.summary,
    ~onGenerated: unit => unit,
  ) => {
    let (documentType, setDocumentType) = React.useState(() => "entrustment")
    // Prefilled from the workplace, still editable for this one document
    let (shortName, setShortName) = React.useState(() =>
      detail.workplaceShortName->Belt.Option.getWithDefault("")
    )
    let (meetingDate, setMeetingDate) = React.useState(() => None)
    let (companyAddress, setCompanyAddress) = React.useState(() =>
      detail.companyAddress->Belt.Option.getWithDefault("")
    )
    let (companyId, setCompanyId) = React.useState(() =>
      detail.companyId->Belt.Option.getWithDefault("")
    )
    let (removedMemberIds, setRemovedMemberIds) = React.useState((): array<string> => [])
    let (removedMemberInput, setRemovedMemberInput) = React.useState(() => "")
    let (error, setError) = React.useState(() => None)

    /* Members who left the committee can't be told apart from the database,
       so their UUIDs are entered by hand. Orca loads their details. */
    let addRemovedMember = _ => {
      let id = removedMemberInput->Js.String2.trim->Js.String2.toLowerCase
      if !isUuid(id) {
        setError(_ => Some("Not a valid member UUID"))
      } else {
        setError(_ => None)
        setRemovedMemberIds(ids => ids->Js.Array2.includes(id) ? ids : Belt.Array.concat(ids, [id]))
        setRemovedMemberInput(_ => "")
      }
    }
    let removeRemovedMember = id =>
      setRemovedMemberIds(ids => ids->Belt.Array.keep(other => other != id))

    let isEntrustment = documentType == "entrustment"
    // Member confirmation only needs data Orca already has
    let needsEmployer = isEntrustment || documentType == "workplace_announcement"

    let onSubmit = _ => {
      let employerInputs = if needsEmployer {
        [
          ("company_address", Json.Encode.string(companyAddress)),
          ("company_id", Json.Encode.string(companyId)),
        ]
      } else {
        []
      }
      let meetingInputs = if isEntrustment {
        [
          (
            "meeting_date",
            Json.Encode.option(Json.Encode.string, meetingDate->Belt.Option.map(localDay)),
          ),
          ("removed_member_ids", Json.Encode.stringArray(removedMemberIds)),
        ]
      } else {
        []
      }
      let body = Json.Encode.object(
        Belt.Array.concatMany([
          [
            ("document_type", Json.Encode.string(documentType)),
            ("workplace_short_name", Json.Encode.string(shortName)),
          ],
          employerInputs,
          meetingInputs,
        ]),
      )
      let req =
        api->Api.postJson(
          ~path="/workplaces/" ++ Uuid.toString(detail.id) ++ "/files",
          ~decoder=Api.Decode.acceptedResponse,
          ~body,
        )
      req->Future.get(res => {
        switch res {
        | Ok(_) => {
            onGenerated()
            Modal.Interface.closeModal(modal)
          }
        | Error(e) => setError(_ => Some(e->Api.showError))
        }
      })
    }

    let selectDocumentType = (event: JsxEvent.Form.t) => {
      let value = ReactEvent.Form.currentTarget(event)["value"]
      setDocumentType(_ => value)
    }

    <Form onSubmit>
      <Form.Field>
        <Form.Label>
          {React.string("Document")}
          <select value=documentType onChange=selectDocumentType>
            {documentTypes
            ->Belt.Array.map(t => <option key=t.value value=t.value> {React.string(t.label)} </option>)
            ->React.array}
          </select>
        </Form.Label>
      </Form.Field>
      {if needsEmployer {
        <Form.Field>
          <Form.Label>
            {React.string("Employer")}
            <div> {React.string(detail.name)} </div>
          </Form.Label>
        </Form.Field>
      } else {
        <p> {React.string("Lists the workplace representatives with their dates of birth.")} </p>
      }}
      // Every document's file name includes it
      <Form.TextField
        label="Workplace Short Name"
        placeholder="ACME CZ"
        value=shortName
        onInput={v => setShortName(_ => v)}
      />
      {if isEntrustment {
        <>
          <Form.Date
            label="Members' Meeting Date"
            value=meetingDate
            onChange={date => setMeetingDate(_ => Some(date))}
          />
          <Form.Field>
            <Form.Label>
              {React.string("Removed Members (optional)")}
              <div className={styles["removedMemberInput"]}>
                <input
                  type_="text"
                  placeholder="Member UUID"
                  value=removedMemberInput
                  onChange={e => {
                    let value = ReactEvent.Form.currentTarget(e)["value"]
                    setRemovedMemberInput(_ => value)
                  }}
                />
                <Button onClick=addRemovedMember> {React.string("Add")} </Button>
              </div>
            </Form.Label>
            {if Belt.Array.length(removedMemberIds) > 0 {
              <ul className={styles["removedMembers"]}>
                {removedMemberIds
                ->Belt.Array.map(id =>
                  <li key=id>
                    <span> {React.string(id)} </span>
                    <Button variant=Button.Danger onClick={_ => removeRemovedMember(id)}>
                      {React.string("Remove")}
                    </Button>
                  </li>
                )
                ->React.array}
              </ul>
            } else {
              React.null
            }}
          </Form.Field>
        </>
      } else {
        React.null
      }}
      {if needsEmployer {
        <>
          <Form.TextField
            label="Company Address"
            placeholder="Vinohradská 1234/56, 120 00 Praha 2"
            value=companyAddress
            onInput={v => setCompanyAddress(_ => v)}
          />
          <Form.TextField
            label="Company ID (IČ)"
            placeholder="12345678"
            value=companyId
            onInput={v => setCompanyId(_ => v)}
          />
        </>
      } else {
        React.null
      }}
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Generate")} </Button>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(msg) => <Message.Error> {React.string(msg)} </Message.Error>
      }}
    </Form>
  }
}

let generateDocumentModal = (~api, ~modal, ~detail, ~onGenerated): Modal.modalContent => {
  title: "Generate Document",
  content: <GenerateDocument api modal detail onGenerated />,
}

let statusButtons = (
  detail: Api.webData<WorkplaceData.summary>,
  ~onEstablish,
  ~onAnnounce,
  ~onCancel,
  ~onGenerateDocument,
) => {
  let generateButton =
    <Button onClick=onGenerateDocument> {React.string("Generate Document")} </Button>

  switch detail {
  | Success(d) =>
    switch WorkplaceData.getStatus(d) {
    | Initial =>
      <Button.Panel>
        <Button variant=Button.Cta onClick=onEstablish> {React.string("Establish")} </Button>
      </Button.Panel>
    | Established =>
      <Button.Panel>
        <Button variant=Button.Cta onClick=onAnnounce> {React.string("Announce")} </Button>
        generateButton
        <Button variant=Button.Danger onClick=onCancel> {React.string("Cancel Workplace")} </Button>
      </Button.Panel>
    | Announced =>
      <Button.Panel>
        generateButton
        <Button variant=Button.Danger onClick=onCancel> {React.string("Cancel Workplace")} </Button>
      </Button.Panel>
    | Cancelled => React.null
    }
  | _ => React.null
  }
}

module Representatives = {
  @react.component
  let make = (~api: Api.t, ~id: Uuid.t) => {
    let (representatives, _, _) =
      api->Hook.getData(
        ~path="/workplaces/" ++ Uuid.toString(id) ++ "/representatives",
        ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.member),
      )

    let columns: array<DataTable.column<WorkplaceData.mineMember>> = [
      {
        name: "ID",
        minMax: ("100px", "1fr"),
        view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
      },
      {
        name: "Member Number",
        minMax: ("200px", "1fr"),
        view: r =>
          MemberSummaryTable.viewPaddedNumber(
            r.memberNumber,
            ~isRepresentative=Some(r.isRepresentative),
            (),
          ),
      },
      {
        name: "First Name",
        minMax: ("150px", "1fr"),
        view: r => r.firstName->View.option(React.string),
      },
      {
        name: "Last Name",
        minMax: ("150px", "1fr"),
        view: r => r.lastName->View.option(React.string),
      },
    ]

    <div className={styles["info"]}>
      <DataTable data=representatives columns>
        <p> {React.string("No representatives yet.")} </p>
      </DataTable>
    </div>
  }
}

/* Generated workplace documents (e.g. entrustments). Orca lists them only for
   staff (`list-workplaces`), so the tab is not shown to workplace executives. */
module Files = {
  @react.component
  let make = (~api: Api.t, ~id: Uuid.t) => {
    let (filesData, _, _) =
      api->Hook.getData(
        ~path="/workplaces/" ++ Uuid.toString(id) ++ "/files",
        ~decoder=Json.Decode.array(Data.Decode.file),
      )
    // All files are loaded, only shown a page at a time
    let pageSize = 10
    let (page, setPage) = React.useState(() => 0)

    <DataGrid
      data=filesData
      layout={[
        {
          label: "",
          cells: [
            {
              label: "Files",
              minmax: ("150px", "600px"),
              view: files => {
                let pageCount = Pagination.pageCount(~total=Belt.Array.length(files), ~pageSize)
                let page = Pagination.clamp(page, ~pageCount)
                <>
                  {View.filesTable(~api, ~files=files->Pagination.slice(~page, ~pageSize))}
                  <Pagination page pageCount onChange={p => setPage(_ => p)} />
                </>
              },
            },
          ],
        },
      ]}
    />
  }
}

type tabs =
  | RepresentativesTab
  | FilesTab

module View = {
  @react.component
  let make = (
    ~api: Api.t,
    ~modal: Modal.Interface.t,
    ~detail: Api.webData<WorkplaceData.summary>,
    ~setDetail,
  ) => {
    let (error, setError) = React.useState(() => None)
    let (generating, setGenerating) = React.useState(() => false)
    let tabHandlers = Tabbed.make(RepresentativesTab)

    let session = React.useContext(SessionContext.context)
    let isStaff =
      session->RemoteData.unwrap(~default=false, s =>
        Session.hasRole(s, ~role=Session.ListWorkplaces)
      )

    let doTransition = (path, _) => {
      let req =
        api->Api.patchJson(
          ~path,
          ~decoder=WorkplaceData.Decode.summary,
          ~body=Json.Encode.object([]),
        )
      req->Future.get(res => {
        switch res {
        | Ok(updated) => {
            setError(_ => None)
            setDetail(_ => RemoteData.Success(updated))
          }
        | Error(e) => setError(_ => Some(e))
        }
      })
    }

    let basePath = switch detail {
    | Success(d) => "/workplaces/" ++ Uuid.toString(d.id)
    | _ => ""
    }

    let openEditModal = _ =>
      switch detail {
      | Success(d) =>
        Modal.Interface.openModal(
          modal,
          editWorkplaceModal(~api, ~modal, ~id=d.id, ~detail=d, ~setDetail),
        )
      | _ => ()
      }

    let openGenerateDocumentModal = _ =>
      switch detail {
      | Success(d) =>
        Modal.Interface.openModal(
          modal,
          generateDocumentModal(~api, ~modal, ~detail=d, ~onGenerated=() =>
            setGenerating(_ => true)
          ),
        )
      | _ => ()
      }

    <Page requireAnyRole=[ListWorkplaces, ListOwnWorkplace] mainResource=detail>
      <header className={styles["header"]}>
        <h1 className={styles["title"]}>
          <span className={styles["titleText"]}>
            {React.string("Workplace ")}
            <span className={styles["titleId"]}>
              {switch detail {
              | Success(d) => d.id->Uuid.toString->React.string
              | _ => React.string("...")
              }}
            </span>
          </span>
          <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
            <Button onClick=openEditModal> {React.string("Edit")} </Button>
          </SessionContext.RequireRole>
        </h1>
        <div className={styles["headerNav"]}>
          <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
            <Page.BackButton name="workplaces" path="/workplaces" />
          </SessionContext.RequireRole>
          <SessionContext.RequireRole anyOf=[Session.ListWorkplaces]>
            <Page.BackButton name="workplace members" path={basePath ++ "/members"} />
          </SessionContext.RequireRole>
          <SessionContext.RequireRole anyOf=[Session.ListOwnWorkplaceMembers]>
            <a
              className={styles["navLink"]}
              onClick={_ => RescriptReactRouter.push("/my-workplace")}>
              {React.string("Show workplace members")}
            </a>
          </SessionContext.RequireRole>
          <SessionContext.RequireRole anyOf=[Session.ListOwnWorkplaceMembers]>
            <a
              className={styles["navLink"]}
              onClick={_ => RescriptReactRouter.push("/my-workplace-missing-dues")}>
              {React.string("Show members with missing payments")}
            </a>
          </SessionContext.RequireRole>
        </div>
      </header>
      <div className={styles["info"]}>
        <DataGrid layout data=detail />
      </div>
      <Tabbed.Tabs>
        <Tabbed.Tab value=RepresentativesTab handlers=tabHandlers>
          {React.string("Representatives")}
        </Tabbed.Tab>
        {if isStaff {
          <Tabbed.Tab value=FilesTab handlers=tabHandlers> {React.string("Files")} </Tabbed.Tab>
        } else {
          React.null
        }}
      </Tabbed.Tabs>
      {switch detail {
      | Success(d) =>
        <>
          <Tabbed.Content tab=RepresentativesTab handlers=tabHandlers>
            <Representatives api id=d.id />
          </Tabbed.Content>
          {if isStaff {
            <Tabbed.Content tab=FilesTab handlers=tabHandlers>
              <Files api id=d.id />
            </Tabbed.Content>
          } else {
            React.null
          }}
        </>
      | _ => React.null
      }}
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
      {if generating {
        <Message.Info>
          {React.string(
            "The document is being generated. It will appear in the Files tab in a moment.",
          )}
        </Message.Info>
      } else {
        React.null
      }}
      <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
        {statusButtons(
          detail,
          ~onEstablish=doTransition(basePath ++ "/establish"),
          ~onAnnounce=doTransition(basePath ++ "/announce"),
          ~onCancel=doTransition(basePath ++ "/cancel"),
          ~onGenerateDocument=openGenerateDocumentModal,
        )}
      </SessionContext.RequireRole>
    </Page>
  }
}

@react.component
let make = (~id: Uuid.t, ~api: Api.t, ~modal: Modal.Interface.t) => {
  let (detail, setDetail, _) =
    api->Hook.getData(
      ~path="/workplaces/" ++ Uuid.toString(id),
      ~decoder=WorkplaceData.Decode.summary,
    )

  <View api modal detail setDetail />
}
