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
        view: d => d.memberCount->Int.toString->React.string,
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
          d.keycloakExecutiveGroupId->Option.mapOr(React.string("—"), uuid =>
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
          | Some(id) => id->Int.toString->React.string
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
        view: d => d.createdAt->Date.toLocaleDateString->React.string,
        minmax: ("150px", "300px"),
      },
      {
        label: "Established On",
        view: d =>
          d.establishedAt->Option.mapOr(React.string("—"), date =>
            date->Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Announced On",
        view: d =>
          d.announcedAt->Option.mapOr(React.string("—"), date =>
            date->Date.toLocaleDateString->React.string
          ),
        minmax: ("150px", "300px"),
      },
      {
        label: "Cancelled On",
        view: d =>
          d.cancelledAt->Option.mapOr(React.string("—"), date =>
            date->Date.toLocaleDateString->React.string
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
      detail.newsletterId->Option.mapOr("", x => Int.toString(x))
    )
    let (shortName, setShortName) = React.useState(_ => detail.workplaceShortName->Option.getOr(""))
    let (companyId, setCompanyId) = React.useState(_ => detail.companyId->Option.getOr(""))
    let (companyAddress, setCompanyAddress) = React.useState(_ =>
      detail.companyAddress->Option.getOr("")
    )
    let (error, setError) = React.useState(() => None)

    /* Orca keeps the stored value when a field is sent empty, so a value once set
     can be changed but not removed. Say so instead of silently keeping it. */
    let clearedField = {
      let cleared = (isStored: bool, input: string) => isStored && input->String.trim == ""
      if cleared(detail.newsletterId->Option.isSome, newsletterId) {
        Some("Newsletter ID")
      } else if cleared(detail.workplaceShortName->Option.isSome, shortName) {
        Some("Short Name")
      } else if cleared(detail.companyId->Option.isSome, companyId) {
        Some("Company ID")
      } else if cleared(detail.companyAddress->Option.isSome, companyAddress) {
        Some("Company Address")
      } else {
        None
      }
    }

    let text = (value: string) => {
      let trimmed = value->String.trim
      trimmed == "" ? Json.Encode.null : Json.Encode.string(trimmed)
    }

    let onSubmit = _ => {
      let parsedId = switch newsletterId->String.trim {
      | "" => Ok(None)
      | s =>
        switch Int.fromString(s) {
        | Some(n) => Ok(Some(n))
        | None => Error("Newsletter ID must be a number")
        }
      }
      switch (parsedId, clearedField) {
      | (Error(msg), _) => setError(_ => Some(msg))
      | (_, Some(field)) => setError(_ => Some(field ++ " can be changed, but not removed"))
      | (Ok(parsedNewsletterId), None) =>
        let body = Json.Encode.object([
          ("newsletter_id", Json.Encode.option(Json.Encode.int)(parsedNewsletterId)),
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
        label="Short Name" placeholder="ACME CZ" value=shortName onInput={v => setShortName(_ => v)}
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
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}
        >
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
let localDay = (date: Date.t): string => {
  let pad = n => (n < 10 ? "0" : "") ++ Int.toString(n)
  Int.toString(Date.getFullYear(date)) ++
  "-" ++
  pad(Date.getMonth(date) + 1) ++
  "-" ++
  pad(Date.getDate(date))
}

let isUuid = (s: string): bool =>
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i->RegExp.test(s)

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
      detail.workplaceShortName->Option.getOr("")
    )
    let (meetingDate, setMeetingDate) = React.useState(() => None)
    let (signatureDate, setSignatureDate) = React.useState(() => Some(Date.make()))
    let (companyAddress, setCompanyAddress) = React.useState(() =>
      detail.companyAddress->Option.getOr("")
    )
    let (companyId, setCompanyId) = React.useState(() => detail.companyId->Option.getOr(""))
    let (removedMemberIds, setRemovedMemberIds) = React.useState((): array<string> => [])
    let (removedMemberInput, setRemovedMemberInput) = React.useState(() => "")
    let (error, setError) = React.useState(() => None)

    let isEntrustment = documentType == "entrustment"

    /* Members who left the committee can't be told apart from the database,
       so they are picked by hand; only their IDs go to Orca, which loads their
       details. Any member can have left, so the list isn't limited to current
       ones. It needs list-members, which a workplace manager may lack -- then
       the IDs are pasted and added by hand instead. */
    let (members, _, _) =
      api->Hook.getData(
        ~enabled=isEntrustment,
        ~path="/members",
        ~decoder=Json.Decode.array(MemberData.Decode.summary),
      )
    let loadedMembers = members->RemoteData.unwrap(~default=[], ms => ms)
    let membersLoaded = switch members {
    | Success(_) => true
    | _ => false
    }

    let addRemovedMember = id => {
      setError(_ => None)
      setRemovedMemberIds(ids => ids->Array.includes(id) ? ids : Array.concat(ids, [id]))
      setRemovedMemberInput(_ => "")
    }

    // A pick from the list (or a pasted ID of a known member) is added at once.
    let onRemovedMemberInput = value =>
      switch MemberSearch.findById(loadedMembers, value) {
      | Some(m) => addRemovedMember(Uuid.toString(m.id))
      | None => setRemovedMemberInput(_ => value)
      }

    // Only offered while the member list isn't available.
    let addPastedMember = _ => {
      let id = removedMemberInput->String.trim->String.toLowerCase
      if isUuid(id) {
        addRemovedMember(id)
      } else {
        setError(_ => Some("Not a valid member UUID"))
      }
    }

    let removeRemovedMember = id =>
      setRemovedMemberIds(ids => ids->Array.filter(other => other != id))

    let viewRemovedMember = id =>
      switch MemberSearch.findById(loadedMembers, id) {
      | Some(m) =>
        <span>
          {MemberSummaryTable.viewPaddedNumber(m.memberNumber, ())}
          {React.string(
            " " ++ [m.firstName, m.lastName]->Array.filterMap(x => x)->Array.joinUnsafe(" "),
          )}
        </span>
      | None => <span className={styles["removedMemberId"]}> {React.string(id)} </span>
      }
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
            Json.Encode.option(Json.Encode.string)(meetingDate->Option.map(localDay)),
          ),
          ("removed_member_ids", Json.Encode.stringArray(removedMemberIds)),
        ]
      } else {
        []
      }
      let body = Json.Encode.object(
        Array.flat([
          [
            ("document_type", Json.Encode.string(documentType)),
            ("workplace_short_name", Json.Encode.string(shortName)),
            (
              "signature_date",
              Json.Encode.option(Json.Encode.string)(signatureDate->Option.map(localDay)),
            ),
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
            ->Array.map(t => <option key=t.value value=t.value> {React.string(t.label)} </option>)
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
        required=true
        value=shortName
        onInput={v => setShortName(_ => v)}
      />
      {if isEntrustment {
        <>
          <Form.Date
            label="Members' Meeting Date"
            required=true
            value=meetingDate
            onChange={date => setMeetingDate(_ => Some(date))}
          />
          <Form.Field>
            <Form.Label>
              {React.string("Removed Members (optional; name, number or ID)")}
              <div className={styles["removedMemberInput"]}>
                <MemberSearch
                  listId="removed-members"
                  members=loadedMembers
                  value=removedMemberInput
                  onChange=onRemovedMemberInput
                />
                {if membersLoaded {
                  React.null
                } else {
                  <Button onClick=addPastedMember> {React.string("Add")} </Button>
                }}
              </div>
            </Form.Label>
            // A valid ID that matched nobody; a matching one was already added.
            {if membersLoaded && isUuid(String.trim(removedMemberInput)) {
              <Message.Warning>
                {React.string("No member with this ID. Pick one from the list.")}
              </Message.Warning>
            } else {
              React.null
            }}
            {if Array.length(removedMemberIds) > 0 {
              <ul className={styles["removedMembers"]}>
                {removedMemberIds
                ->Array.map(id =>
                  <li key=id>
                    {viewRemovedMember(id)}
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
      <Form.Date
        label="Date of Signature"
        required=true
        value=signatureDate
        // the picker is clearable and then passes null
        onChange={date => setSignatureDate(_ => Nullable.make(date)->Nullable.toOption)}
      />
      {if needsEmployer {
        <>
          <Form.TextField
            label="Company Address"
            placeholder="Vinohradská 1234/56, 120 00 Praha 2"
            required=true
            value=companyAddress
            onInput={v => setCompanyAddress(_ => v)}
          />
          <Form.TextField
            label="Company ID (IČ)"
            placeholder="12345678"
            required=true
            value=companyId
            onInput={v => setCompanyId(_ => v)}
          />
        </>
      } else {
        React.null
      }}
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}
        >
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
                let pageCount = Pagination.pageCount(~total=Array.length(files), ~pageSize)
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
              className={styles["navLink"]} onClick={_ => RescriptReactRouter.push("/my-workplace")}
            >
              {React.string("Show workplace members")}
            </a>
          </SessionContext.RequireRole>
          <SessionContext.RequireRole anyOf=[Session.ListOwnWorkplaceMembers]>
            <a
              className={styles["navLink"]}
              onClick={_ => RescriptReactRouter.push("/my-workplace-missing-dues")}
            >
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
          ~onEstablish=doTransition(basePath ++ "/establish", ...),
          ~onAnnounce=doTransition(basePath ++ "/announce", ...),
          ~onCancel=doTransition(basePath ++ "/cancel", ...),
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
