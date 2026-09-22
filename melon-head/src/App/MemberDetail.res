@module external styles: {..} = "./MemberDetail/styles.module.scss"

open Data
open Belt

// todo: we should make Note editable also in application detail
// https://github.com/ictunion/main-system/issues/164
let layout: DataGrid.t<MemberData.detail> = [
  {
    label: "Membership",
    cells: [
      {
        label: "Member Number",
        view: d => MemberSummaryTable.viewPaddedNumber(d.memberNumber, ()),
        minmax: ("250px", "690px"),
      },
      {
        label: "Language",
        view: d => View.option(d.language, React.string),
        minmax: ("200px", "200px"),
      },
      {
        label: "Application",
        view: d =>
          View.option(d.applicationId, uuid =>
            <a onClick={_ => RescriptReactRouter.push("/applications/" ++ Uuid.toString(uuid))}>
              {React.string(uuid->Uuid.toString)}
            </a>
          ),
        minmax: ("250px", "655px"),
      },
    ],
  },
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
      {
        label: "City",
        view: d => View.option(d.city, React.string),
        minmax: ("150px", "500px"),
      },
      {
        label: "Postal Code",
        view: d => View.option(d.postalCode, React.string),
        minmax: ("150px", "150px"),
      },
    ],
  },
  {
    label: "Notes",
    cells: [
      {
        label: "Note",
        view: d => View.option(d.note, React.string),
        minmax: ("150px", "1500px"),
      },
    ],
  },
]

/* What a workplace executive committee member sees of one of their colleagues.

   Orca redacts date of birth, address, city, postal code and the staff note
   before they ever leave the server (see `redact_for_workplace_executive`), so
   this is not a security boundary -- it just avoids rendering rows of blanks. */
let workplaceExecutiveLayout: DataGrid.t<MemberData.detail> = [
  {
    label: "Membership",
    cells: [
      {
        label: "Member Number",
        view: d => MemberSummaryTable.viewPaddedNumber(d.memberNumber, ()),
        minmax: ("250px", "690px"),
      },
      {
        label: "Language",
        view: d => View.option(d.language, React.string),
        minmax: ("200px", "200px"),
      },
    ],
  },
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
]

let timeRows: array<RowBasedTable.row<MemberData.detail>> = [
  ("Created", d => d.createdAt->Js.Date.toLocaleString->React.string),
  (
    "Onboarded at",
    d => View.option(d.onboardingFinishAt, a => a->Js.Date.toLocaleString->React.string),
  ),
  ("Left at", d => View.option(d.leftAt, a => a->Js.Date.toLocaleString->React.string)),
]

module Loading = {
  @react.component
  let make = () => {
    React.string("loading...")
  }
}

module Actions = {
  open MemberData

  module Accept = {
    type acceptTabs =
      | Create
      | Pair

    @react.component
    let make = (~modal, ~api, ~id, ~setDetail) => {
      let (error, setError) = React.useState(() => None)
      let tabHandlers = Tabbed.make(Create)
      let doAccept = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/accept",
            ~decoder=MemberData.Decode.detail,
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

      let (selectedId, setId) = React.useState(_ => None)

      let selectId = (event: JsxEvent.Form.t) => {
        let newVal = ReactEvent.Form.currentTarget(event)["value"]
        setId(_ => Some(newVal))
      }

      let doPair = (_: JsxEvent.Mouse.t) => {
        let uuid = switch selectedId {
        | Some(uuid) => Json.Encode.string(uuid)
        | None => Json.Encode.null
        }
        let req =
          api->Api.patchJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/pair_oid",
            ~decoder=MemberData.Decode.detail,
            ~body=Json.Encode.object([("sub", uuid)]),
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

      let (candidates: Api.webData<array<Session.user>>, _, _) =
        api->Hook.getData(
          ~path="/members/" ++ Uuid.toString(id) ++ "/list_candidate_users",
          ~decoder=Json.Decode.array(Session.Decode.user),
        )

      <Modal.Content>
        <Tabbed.Tabs>
          <Tabbed.Tab value=Create handlers=tabHandlers> {React.string("Create")} </Tabbed.Tab>
          <Tabbed.Tab value=Pair handlers=tabHandlers> {React.string("Pair Existing")} </Tabbed.Tab>
        </Tabbed.Tabs>
        <Tabbed.Content tab=Create handlers=tabHandlers>
          <div className={styles["modalBody"]}>
            <p> {React.string("Accept member and allow them to access internal resources.")} </p>
            {switch error {
            | None => React.null
            | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
            }}
          </div>
          <Button.Panel>
            <Button onClick={_ => modal->Modal.Interface.closeModal}>
              {React.string("Cancel")}
            </Button>
            <Button variant=Button.Cta onClick=doAccept> {React.string("Accept member")} </Button>
          </Button.Panel>
        </Tabbed.Content>
        <Tabbed.Content tab=Pair handlers=tabHandlers>
          <div className={styles["modalBody"]}>
            <p> {React.string("Pair existing OID account with member")} </p>
            {switch candidates {
            | Idle => <Loading />
            | Loading => <Loading />
            | Failure(err) => React.string(Api.showError(err))
            | Success(candidates) =>
              if candidates == [] {
                React.string("No candidates found")
              } else {
                <div className={styles["radioList"]}>
                  {candidates
                  ->Array.map(user => {
                    <label className={styles["radio"]}>
                      <input value={user.id->Uuid.toString} type_="radio" onInput=selectId />
                      {React.string(user.email->Data.Email.toString)}
                    </label>
                  })
                  ->React.array}
                </div>
              }
            }}
            {switch error {
            | None => React.null
            | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
            }}
          </div>
          <Button.Panel>
            <Button onClick={_ => modal->Modal.Interface.closeModal}>
              {React.string("Cancel")}
            </Button>
            <Button variant=Button.Cta onClick=doPair disabled={selectedId == None}>
              {React.string("Pair Selected")}
            </Button>
          </Button.Panel>
        </Tabbed.Content>
      </Modal.Content>
    }
  }

  module Remove = {
    @react.component
    let make = (~modal, ~api, ~id, ~setDetail) => {
      let (error, setError) = React.useState(() => None)

      let doRemove = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.deleteJson(
            ~path="/members/" ++ Uuid.toString(id),
            ~decoder=MemberData.Decode.detail,
            ~body=None,
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
        <p> {React.string("Remove member and reject their access to organization resources.")} </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => modal->Modal.Interface.closeModal}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Danger onClick=doRemove> {React.string("Remove Member")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  let acceptModal = (~modal, ~api, ~id, ~setDetail): Modal.modalContent => {
    title: "Accept Member",
    content: <Accept modal api id setDetail />,
  }

  let removeModal = (~modal, ~api, ~id, ~setDetail): Modal.modalContent => {
    title: "Remove Member",
    content: <Remove modal api id setDetail />,
  }

  @react.component
  let make = (~status, ~modal, ~api, ~id, ~setDetail, ~hasSub) => {
    let (oidError, setOidError) = React.useState(() => None)
    let (oidScheduled, setOidScheduled) = React.useState(() => false)

    let doCreateOidAccount = (_: JsxEvent.Mouse.t) => {
      let req =
        api->Api.postJson(
          ~path="/members/" ++ Uuid.toString(id) ++ "/create_oid_account",
          ~decoder=Api.Decode.acceptedResponse,
          ~body=Js.Json.null,
        )
      req->Future.get(res => {
        switch res {
        | Ok(_) => setOidScheduled(_ => true)
        | Error(e) => setOidError(_ => Some(e))
        }
      })
    }

    let createOidButton = if hasSub || oidScheduled {
      React.null
    } else {
      <Button variant=Button.Cta onClick=doCreateOidAccount>
        {React.string("Create Keycloak Account")}
      </Button>
    }

    <>
      {switch oidError {
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | None => React.null
      }}
      {switch status {
      | NewMember =>
        <Button.Panel>
          <Button
            variant=Button.Cta
            onClick={_ => RescriptReactRouter.push("/members/" ++ Uuid.toString(id) ++ "/welcome")}>
            {React.string("Send welcome email")}
          </Button>
          <Button
            variant=Button.Cta
            onClick={_ =>
              RescriptReactRouter.push("/members/" ++ Uuid.toString(id) ++ "/workplacewelcome")}>
            {React.string("Send workplace welcome email")}
          </Button>
          <Button
            variant=Button.Cta
            onClick={_ =>
              modal->Modal.Interface.openModal(acceptModal(~modal, ~api, ~id, ~setDetail))}>
            {React.string("Accept member")}
          </Button>
          createOidButton
          <Button
            variant=Button.Danger
            onClick={_ =>
              modal->Modal.Interface.openModal(removeModal(~modal, ~api, ~id, ~setDetail))}>
            {React.string("Remove member")}
          </Button>
        </Button.Panel>
      | CurrentMember =>
        <Button.Panel>
          createOidButton
          <Button
            variant=Button.Danger
            onClick={_ =>
              modal->Modal.Interface.openModal(removeModal(~modal, ~api, ~id, ~setDetail))}>
            {React.string("Remove member")}
          </Button>
        </Button.Panel>
      | PastMember => React.null
      }}
    </>
  }
}

module MemberWorkplaceSelect = {
  module Loading = {
    @react.component
    let make = () => <>
      <dt> {React.string("Workplace:")} </dt>
      <dd>
        <select disabled={true}>
          <option> {React.string("(loading...)")} </option>
        </select>
      </dd>
    </>
  }

  module Active = {
    let viewWorkplaces = (workplace: WorkplaceData.summary) => {
      <option key={workplace.id->Uuid.toString} value={workplace.id->Uuid.toString}>
        {React.string(workplace.name)}
      </option>
    }

    @react.component
    let make = (~api, ~detail: MemberData.detail, ~workplaces: array<WorkplaceData.summary>) => {
      let (workplaceId, setWorkplaceId) = React.useState(() => detail.workplaceId)
      // Assigning to a workplace (whether the plain PUT below or switching away
      // entirely) always clears representative status server-side -- mirror
      // that here so the checkbox never shows stale state for a workplace the
      // member was just moved out of.
      let (isRepresentative, setIsRepresentative) = React.useState(() =>
        detail.isRepresentative->Option.getWithDefault(false)
      )

      let onWorkplaceChange = e => {
        let value = ReactEvent.Form.currentTarget(e)["value"]

        // remove current workplace -> workplaceID that is assigned before this change event
        let _ = workplaceId->Option.map(workplaceId => {
          let _ =
            api->Api.deleteJson(
              ~path="/workplaces/" ++
              Uuid.toString(workplaceId) ++
              "/members/" ++
              Uuid.toString(detail.id),
              ~decoder=Api.Decode.acceptedResponse,
              ~body=None,
            )
        })

        // get selected/new workplace ID or None
        let newWorkplaceId = switch value {
        | "" => None
        | v => Some(Uuid.unsafeFromString(v))
        }

        // add new workplace if user selected valid workplace from menu
        let _ =
          newWorkplaceId->Option.map(newWorkplaceId =>
            api->Api.put(
              ~path="/workplaces/" ++
              Uuid.toString(newWorkplaceId) ++
              "/members/" ++
              Uuid.toString(detail.id),
              ~decoder=Api.Decode.acceptedResponse,
            )
          )

        setWorkplaceId(_ => newWorkplaceId)
        setIsRepresentative(_ => false)
      }

      let onRepresentativeChange = (workplaceId, e) => {
        let checked = ReactEvent.Form.currentTarget(e)["checked"]

        let path = if checked {
          "/workplaces/" ++
          Uuid.toString(workplaceId) ++
          "/members/" ++
          Uuid.toString(detail.id) ++ "/is_representative"
        } else {
          "/workplaces/" ++ Uuid.toString(workplaceId) ++ "/members/" ++ Uuid.toString(detail.id)
        }

        let _ = api->Api.put(~path, ~decoder=Api.Decode.acceptedResponse)

        setIsRepresentative(_ => checked)
      }

      <>
        <dt> {React.string("Workplace:")} </dt>
        <dd>
          <select
            defaultValue={switch detail.workplaceId {
            | Some(id) => Uuid.toString(id)
            | None => ""
            }}
            onChange=onWorkplaceChange>
            <option value=""> {React.string("(none)")} </option>
            {workplaces->Array.map(viewWorkplaces)->React.array}
          </select>
        </dd>
        {switch workplaceId {
        | Some(wid) =>
          <>
            <dt> {React.string("Representative:")} </dt>
            <dd>
              <label>
                <input
                  type_="checkbox" checked=isRepresentative onChange={onRepresentativeChange(wid)}
                />
              </label>
            </dd>
          </>
        | None => React.null
        }}
      </>
    }
  }

  @react.component
  let make = (~api, ~detail: Api.webData<MemberData.detail>) => {
    let (workplaces, _, _) =
      api->Hook.getData(
        ~path="/workplaces",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )

    {
      switch (detail, workplaces) {
      | (Success(detail), Success(workplaces)) => <Active api detail workplaces />
      | _ => <Loading />
      }
    }
  }
}

module PaymentsTab = {
  module Intl = {
    type dateTimeFormat

    @new @scope("Intl")
    external makeDateTimeFormat: (Js.Nullable.t<string>, {"month": string}) => dateTimeFormat =
      "DateTimeFormat"

    @send external format: (dateTimeFormat, Js.Date.t) => string = "format"
  }

  // Uses the browser's locale (Intl default), so month names aren't hardcoded to English.
  let monthFormatter = Intl.makeDateTimeFormat(Js.Nullable.undefined, {"month": "long"})

  let monthNames = Array.makeBy(12, month =>
    Intl.format(
      monthFormatter,
      Js.Date.makeWithYM(~year=2000.0, ~month=Belt.Int.toFloat(month), ()),
    )
  )

  let yearOf = (date: Js.Date.t): int => date->Js.Date.getFullYear->Float.toInt
  let monthOf = (date: Js.Date.t): int => date->Js.Date.getMonth->Float.toInt + 1

  let findTransaction = (transactions: array<PaymentData.transaction>, ~year, ~month) =>
    transactions->Array.getBy(t =>
      t.coveredMonths->Array.some(cm => cm.year == year && cm.month == month)
    )

  /* One transaction can cover several months at once (a member catching up
     on arrears). We show the amount only on the most recent of those months
     and point the other rows at it, instead of repeating the same amount on
     every covered row. */
  let mostRecentCoveredMonth = (t: PaymentData.transaction): option<PaymentData.coveredMonth> =>
    t.coveredMonths->Array.reduce(None, (acc, cm) =>
      switch acc {
      | None => Some(cm)
      | Some(best) if cm.year > best.year || (cm.year == best.year && cm.month > best.month) =>
        Some(cm)
      | Some(_) as best => best
      }
    )

  let viewAmount = (t: PaymentData.transaction) => t.amount ++ " " ++ t.currency

  /* A member owes dues starting the month they joined, and a given month's
     due is paid the month after it (a payment landing in September covers
     August) -- so the current calendar month, and anything after it, can
     never have a payment yet and shouldn't read as a missing payment. A real
     transaction match always wins over both of those, though, in case a
     member paid ahead. */
  type cellStatus =
    | NotMember
    | Future
    | Paid(string)
    | PaidElsewhere(PaymentData.coveredMonth)
    | Missing
    | LoadingCell

  let cellStatus = (
    ~year,
    ~month,
    ~joinYear,
    ~joinMonth,
    ~currentYear,
    ~currentMonth,
    ~paymentsData: Api.webData<array<PaymentData.transaction>>,
  ) => {
    let transactionMatch = switch paymentsData {
    | Success(transactions) => findTransaction(transactions, ~year, ~month)
    | Idle
    | Loading
    | Failure(_) => None
    }

    switch transactionMatch {
    | Some(t) =>
      switch mostRecentCoveredMonth(t) {
      | Some(recent) if recent.year == year && recent.month == month => Paid(viewAmount(t))
      | Some(recent) => PaidElsewhere(recent)
      | None => Paid(viewAmount(t))
      }
    | None =>
      if year < joinYear || (year == joinYear && month < joinMonth) {
        NotMember
      } else if year > currentYear || (year == currentYear && month >= currentMonth) {
        Future
      } else {
        switch paymentsData {
        | Loading => LoadingCell
        | Success(_)
        | Idle
        | Failure(_) => Missing
        }
      }
    }
  }

  let viewCellStatus = (status: cellStatus) =>
    switch status {
    | NotMember => <span className={styles["notDue"]}> {React.string("NOT MEMBER")} </span>
    | Future => <span className={styles["notDue"]}> {React.string("FUTURE")} </span>
    | Paid(amount) => <span> {React.string(amount)} </span>
    | PaidElsewhere(recent) =>
      let monthName = monthNames->Array.get(recent.month - 1)->Option.getWithDefault("")
      <span className={styles["notDue"]}>
        {React.string("Paid in " ++ monthName ++ " " ++ Js.Int.toString(recent.year))}
      </span>
    | Missing => <span> {React.string("---")} </span>
    | LoadingCell => <span> {React.string("...")} </span>
    }

  @react.component
  let make = (~bankApi: Api.t, ~detail: MemberData.detail) => {
    let joinDate = detail.onboardingFinishAt->Option.getWithDefault(detail.createdAt)
    let joinYear = yearOf(joinDate)
    let joinMonth = monthOf(joinDate)
    let now = Js.Date.make()
    let currentYear = now->yearOf
    let currentMonth = now->monthOf

    let years = Array.makeBy(max(currentYear - joinYear + 1, 1), i => joinYear + i)

    let yearHandlers = Tabbed.make(currentYear)

    let (paymentsData: Api.webData<array<PaymentData.transaction>>, _, _) =
      bankApi->Hook.getData(
        ~path="/payments/" ++ Int.toString(detail.memberNumber) ++ "/history",
        ~decoder=PaymentData.Decode.history,
      )

    <div className={styles["payments"]}>
      <Tabbed.Tabs>
        {years
        ->Array.map(year =>
          <Tabbed.Tab key={year->Js.Int.toString} value=year handlers=yearHandlers>
            {React.string(year->Js.Int.toString)}
          </Tabbed.Tab>
        )
        ->React.array}
      </Tabbed.Tabs>
      {switch paymentsData {
      | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | _ => React.null
      }}
      {years
      ->Array.map(year =>
        <Tabbed.Content key={year->Js.Int.toString} tab=year handlers=yearHandlers>
          <table className={styles["paymentsTable"]}>
            <tbody>
              {monthNames
              ->Array.mapWithIndex((idx, name) => {
                let month = idx + 1
                let status = cellStatus(
                  ~year,
                  ~month,
                  ~joinYear,
                  ~joinMonth,
                  ~currentYear,
                  ~currentMonth,
                  ~paymentsData,
                )
                <tr key={idx->Js.Int.toString}>
                  <td> {React.string(name)} </td>
                  <td> {viewCellStatus(status)} </td>
                </tr>
              })
              ->React.array}
            </tbody>
          </table>
        </Tabbed.Content>
      )
      ->React.array}
    </div>
  }
}

type tabs =
  | Metadata
  | Files
  | Occupations
  | Payments
  | Workplace

let viewOccupation = (
  ~api: Api.t,
  ~memberId,
  ~setOccupationsData,
  occupation: MemberData.occupation,
) => {
  let doDelete = _ => {
    let req =
      api->Api.deleteJson(
        ~path="/members/" ++
        Uuid.toString(memberId) ++
        "/occupations/" ++
        Uuid.toString(occupation.id),
        ~decoder=Api.Decode.acceptedResponse,
        ~body=None,
      )

    req->Future.get(res => {
      switch res {
      | Ok(_) =>
        setOccupationsData(prev =>
          prev->RemoteData.map(xs =>
            xs->Array.keep((o: MemberData.occupation) => o.id != occupation.id)
          )
        )
      | Error(_) => ()
      }
    })
  }

  <tr key={occupation.id->Uuid.toString}>
    <td> {occupation.companyName->View.option(React.string)} </td>
    <td> {occupation.position->View.option(React.string)} </td>
    <td> {React.string(occupation.source)} </td>
    <td> {occupation.createdAt->Js.Date.toLocaleDateString->React.string} </td>
    <td>
      {if occupation.source != "application" {
        <SessionContext.RequireRole anyOf=[Session.ManageMembers]>
          <Button variant=Button.Danger onClick=doDelete> {React.string("Delete")} </Button>
        </SessionContext.RequireRole>
      } else {
        React.null
      }}
    </td>
  </tr>
}

module AddOccupation = {
  let emptyOccupation: MemberData.newOccupation = {companyName: "", position: "", source: "orca"}

  let fromCurrent = (current: MemberData.occupation): MemberData.newOccupation => {
    companyName: current.companyName->Option.map(name => "Ex-" ++ name)->Option.getWithDefault(""),
    position: current.position->Option.getWithDefault(""),
    source: "orca",
  }

  @react.component
  let make = (~modal, ~api: Api.t, ~id, ~current: option<MemberData.occupation>, ~setOccupationsData) => {
    let (newOccupation, setNewOccupation) = React.useState(_ =>
      current->Option.mapWithDefault(emptyOccupation, fromCurrent)
    )
    let (error, setError) = React.useState(() => None)

    let onSubmit = _ => {
      let body = MemberData.Encode.newOccupation(newOccupation)
      let req =
        api->Api.postJson(
          ~path="/members/" ++ Uuid.toString(id) ++ "/occupations",
          ~decoder=MemberData.Decode.occupation,
          ~body,
        )

      req->Future.get(res => {
        switch res {
        | Ok(data) => {
            setOccupationsData(prev => prev->RemoteData.map(xs => Array.concat([data], xs)))
            Modal.Interface.closeModal(modal)
          }
        | Error(e) => setError(_ => Some(e))
        }
      })
    }

    <Form onSubmit>
      <Form.TextField
        label="Company Name"
        placeholder="Evil corp."
        value=newOccupation.companyName
        onInput={companyName => setNewOccupation(o => {...o, companyName})}
      />
      <Form.TextField
        label="Position"
        placeholder="Site Reliability Engineer"
        value=newOccupation.position
        onInput={position => setNewOccupation(o => {...o, position})}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" variant=Button.Cta> {React.string("Add Occupation")} </Button>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
    </Form>
  }
}

let addOccupationModal = (~api, ~modal, ~id, ~current, ~setOccupationsData): Modal.modalContent => {
  title: "Add Occupation",
  content: <AddOccupation modal api id current setOccupationsData />,
}

@react.component
let make = (~api, ~bankApi, ~id, ~modal) => {
  let (detail: Api.webData<MemberData.detail>, setDetail, _) =
    api->Hook.getData(~path="/members/" ++ Uuid.toString(id), ~decoder=MemberData.Decode.detail)

  let status = RemoteData.map(detail, MemberData.getStatus)

  /* A workplace executive reaches this page through /my-workplace and holds none
     of the staff roles. Orca serves them a redacted record for members of their
     own workplace only, so the page has to be shown without the staff sections
     rather than not shown at all. */
  let session = React.useContext(SessionContext.context)

  let isStaff =
    session->RemoteData.unwrap(~default=false, s =>
      Session.hasRole(s, ~role=Session.ListMembers) || Session.hasRole(s, ~role=Session.ViewMember)
    )

  /* Payment history stays gated on the payment-history bank role for staff,
     but a workplace executive gets it too -- for their own workplace's
     members only, which Orca already enforced by serving this page at all. */
  let canSeePayments =
    session->RemoteData.unwrap(~default=false, s =>
      Session.hasBankRole(s, ~role=Session.PaymentHistory) ||
      Session.hasRole(s, ~role=Session.ListOwnWorkplaceMembers)
    )

  let tabHandlers = Tabbed.make(isStaff ? Occupations : Payments)

  let (filesData, _, _) =
    api->Hook.getData(
      ~path="/members/" ++ Uuid.toString(id) ++ "/files",
      ~decoder=Json.Decode.array(Data.Decode.file),
    )

  let (occupationsData, setOccupationsData, _) =
    api->Hook.getData(
      ~path="/members/" ++ Uuid.toString(id) ++ "/occupations",
      ~decoder=Json.Decode.array(MemberData.Decode.occupation),
    )

  let mainOccupation = occupationsData->RemoteData.map(xs => xs->Array.get(0))

  let openAddOccupationModal = _ => {
    let current = mainOccupation->RemoteData.toOption->Option.flatMap(x => x)
    modal->Modal.Interface.openModal(
      addOccupationModal(~api, ~modal, ~id, ~current, ~setOccupationsData),
    )
  }

  /* A workplace executive reaches this page through /my-workplace and holds none
     of the staff roles. Orca serves them a redacted record for members of their
     own workplace only, so the page has to be shown without the staff sections
     rather than not shown at all. */
  let session = React.useContext(SessionContext.context)

  let isStaff =
    session
    ->RemoteData.unwrap(~default=false, s =>
      Session.hasRole(s, ~role=Session.ListMembers) || Session.hasRole(s, ~role=Session.ViewMember)
    )

  <Page
    requireAnyRole=[ListMembers, ViewMember, ListOwnWorkplaceMembers] mainResource=detail>
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
      <div className={styles["navButtons"]}>
        <SessionContext.RequireRole anyOf=[Session.ListMembers]>
          <Page.BackButton name="members" path={status->RemoteData.toOption->Members.tabToUrl} />
        </SessionContext.RequireRole>
        <SessionContext.RequireRole anyOf=[Session.ListWorkplaces]>
          <SessionContext.RequireRole anyOf=[Session.ViewMember]>
            {switch detail {
            | Success(d) =>
              switch d.workplaceId {
              | Some(wid) =>
                <Page.BackButton
                  name="workplace members" path={"/workplaces/" ++ Uuid.toString(wid) ++ "/members"}
                />
              | None => React.null
              }
            | _ => React.null
            }}
          </SessionContext.RequireRole>
        </SessionContext.RequireRole>
        {if isStaff {
          React.null
        } else {
          <Page.BackButton name="my workplace" path="/my-workplace" />
        }}
      </div>
      <dl className={styles["headerRow"]}>
        <dt> {React.string("Status:")} </dt>
        <dd>
          <Chip.MemberStatus value=status />
        </dd>
        <SessionContext.RequireRole anyOf=[Session.ManageWorkplaces]>
          <MemberWorkplaceSelect api detail />
        </SessionContext.RequireRole>
      </dl>
    </header>
    <DataGrid layout={isStaff ? layout : workplaceExecutiveLayout} data=detail />
    <DataGrid
      data=mainOccupation
      layout={[
        {
          label: "Last Occupation",
          cells: [
            {
              label: "Company",
              view: d =>
                d
                ->Option.flatMap((a: MemberData.occupation) => a.companyName)
                ->View.option(React.string),
              minmax: ("150px", "900px"),
            },
            {
              label: "Position",
              view: d => d->Option.flatMap(a => a.position)->View.option(React.string),
              minmax: ("150px", "665px"),
            },
          ],
        },
      ]}
    />
    <Tabbed.Tabs>
      {if isStaff {
        <Tabbed.Tab value=Occupations handlers=tabHandlers>
          {React.string("Occupations")}
        </Tabbed.Tab>
      } else {
        React.null
      }}
      {if canSeePayments {
        <Tabbed.Tab value=Payments handlers=tabHandlers> {React.string("Payments")} </Tabbed.Tab>
      } else {
        React.null
      }}
      {if isStaff {
        <>
          <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
          /* Files are scanned membership applications -- signatures and identity
           documents. Staff only, regardless of workplace scope. */
          <Tabbed.Tab value=Files handlers=tabHandlers> {React.string("Files")} </Tabbed.Tab>
        </>
      } else {
        React.null
      }}
      // <Tabbed.Tab value=Workplace handlers=tabHandlers> {React.string("Workplace")} </Tabbed.Tab>
    </Tabbed.Tabs>
{if isStaff {
    <Tabbed.Content tab=Occupations handlers=tabHandlers>
      <div className={styles["occupations"]}>
        <table>
          <thead>
            <tr>
              <th> {React.string("Company")} </th>
              <th> {React.string("Position")} </th>
              <th> {React.string("Source")} </th>
              <th> {React.string("Created at")} </th>
              <th />
            </tr>
          </thead>
          <tbody>
            {switch occupationsData {
            | Success(occupations) =>
              occupations
              ->Array.map(viewOccupation(~api, ~memberId=id, ~setOccupationsData))
              ->React.array
            | _ => React.null
            }}
          </tbody>
        </table>
        <SessionContext.RequireRole anyOf=[Session.ManageMembers]>
          <div className={styles["occupationsFooter"]}>
            <Button variant=Button.Cta onClick=openAddOccupationModal>
              {React.string("+ Add Occupation")}
            </Button>
          </div>
        </SessionContext.RequireRole>
      </div>
    </Tabbed.Content>
    } else {
      React.null
    }}
    {if canSeePayments {
    <SessionContext.RequireBankRole anyOf=[Session.PaymentHistory]>
      <Tabbed.Content tab=Payments handlers=tabHandlers>
        {switch detail {
        | Success(d) => <PaymentsTab bankApi detail=d />
        | _ => <Loading />
        }}
      </Tabbed.Content>
    </SessionContext.RequireBankRole>
    } else {
      React.null
    }}
    {if isStaff {
      <>
        <Tabbed.Content tab=Metadata handlers=tabHandlers>
          <div className={styles["metadata"]}>
            <RowBasedTable rows=timeRows data=detail title=Some("Updates") />
          </div>
        </Tabbed.Content>
        <Tabbed.Content tab=Files handlers=tabHandlers>
          <DataGrid
            data=filesData
            layout={[
              {
                label: "",
                cells: [
                  {
                    label: "Files",
                    minmax: ("150px,", "600px"),
                    view: files => View.filesTable(~api, ~files),
                  },
                ],
              },
            ]}
          />
        </Tabbed.Content>
      </>
    } else {
      React.null
    }}
    /* Accept / remove / create-account all need staff roles server side; a
     workplace executive would only get a 403 out of them. */
    <SessionContext.RequireRole anyOf=[Session.ManageMembers]>
      {switch (status, detail) {
      | (Success(s), Success(d)) =>
        <Actions status=s modal api id setDetail hasSub={d.sub->Option.isSome} />
      | _ => React.null
      }}
    </SessionContext.RequireRole>
  </Page>
}
