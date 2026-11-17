@module external styles: {..} = "./MissingDues/styles.module.scss"

open Data

/* One member's payment history, fetched when first shown. The join date
   comes from the member detail, since the list summary doesn't carry
   onboarding_finished_at. */
module HistoryPreview = {
  @react.component
  let make = (~api: Api.t, ~bankApi: Api.t, ~member: MemberData.summary) => {
    let (detail, _, _) =
      api->Hook.getData(
        ~path="/members/" ++ Uuid.toString(member.id),
        ~decoder=MemberData.Decode.detail,
      )
    let (paymentsData, _, _) =
      bankApi->Hook.getData(
        ~path="/payments/" ++ Int.toString(member.memberNumber) ++ "/history",
        ~decoder=PaymentData.Decode.history,
      )

    switch detail {
    | Success(d) =>
      <PaymentHistory paymentsData joinDate={PaymentHistory.joinDateOf(d)} leftAt=d.leftAt />
    | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
    | Idle
    | Loading =>
      <div className={styles["loading"]}>
        <Icons.Loading variant=Icons.Dark />
      </div>
    }
  }
}

/* The Missed Months count, which opens the member's payment history: on
   hover for a quick look, or pinned open by a click until closed again (a
   second click, a click outside, Escape or the close button). The table
   scrolls inside its own container, which would clip a nested popover, so it
   is position: fixed and placed from the count's on-screen box instead. */
module MissedMonthsCell = {
  type rect = {left: float, top: float, bottom: float}

  @send external getBoundingClientRect: Dom.element => rect = "getBoundingClientRect"
  @send external contains: (Dom.element, Dom.element) => bool = "contains"
  @get external target: Dom.event => Dom.element = "target"
  @get external key: Dom.event => string = "key"
  @val external innerWidth: float = "innerWidth"
  @val external innerHeight: float = "innerHeight"
  @val @scope("window")
  external addWindowListener: (string, Dom.event => unit, bool) => unit = "addEventListener"
  @val @scope("window")
  external removeWindowListener: (string, Dom.event => unit, bool) => unit = "removeEventListener"

  // Hovering only opens after a short pause, so sweeping the mouse down the
  // column doesn't fire a history request for every row it crosses.
  let openDelay = 300
  let closeDelay = 150

  let margin = 16.0
  let maxWidth = 760.0
  // Below the count unless there's clearly more room above it.
  let minHeightBelow = 320.0

  let px = (n: float) => Float.toString(n) ++ "px"

  let place = (anchor: rect): ReactDOM.Style.t => {
    let width = Math.min(maxWidth, innerWidth -. 2.0 *. margin)
    let left = Math.max(margin, Math.min(anchor.left, innerWidth -. margin -. width))
    let spaceBelow = innerHeight -. anchor.bottom -. margin
    let spaceAbove = anchor.top -. margin

    if spaceBelow >= minHeightBelow || spaceBelow >= spaceAbove {
      {left: px(left), width: px(width), top: px(anchor.bottom), maxHeight: px(spaceBelow)}
    } else {
      {
        left: px(left),
        width: px(width),
        bottom: px(innerHeight -. anchor.top),
        maxHeight: px(spaceAbove),
      }
    }
  }

  let hidden: ReactDOM.Style.t = {display: "none"}

  @react.component
  let make = (~api: Api.t, ~bankApi: Api.t, ~member: MemberData.summary, ~missedMonths: int) => {
    let anchorRef = React.useRef(Nullable.null)
    let popoverRef = React.useRef(Nullable.null)
    let timer = React.useRef(None)
    let (hovered, setHovered) = React.useState(_ => false)
    let (pinned, setPinned) = React.useState(_ => false)
    /* None until first opened. After that the preview stays mounted (just
     hidden) so reopening doesn't fetch the history again. */
    let (placement, setPlacement) = React.useState(_ => None)
    let isOpen = hovered || pinned

    let cancelTimer = () => {
      timer.current->Option.forEach(clearTimeout)
      timer.current = None
    }

    let after = (ms, f) => {
      cancelTimer()
      timer.current = Some(setTimeout(f, ms))
    }

    React.useEffect0(() => Some(cancelTimer))

    let close = () => {
      cancelTimer()
      setPinned(_ => false)
      setHovered(_ => false)
    }

    // Layout effect, so a reopened popover never paints at its old position.
    React.useLayoutEffect1(() =>
      if isOpen {
        let updatePlacement = _ =>
          anchorRef.current
          ->Nullable.toOption
          ->Option.forEach(el => setPlacement(_ => Some(place(el->getBoundingClientRect))))

        let onKeyDown = e =>
          if e->key == "Escape" {
            close()
          }

        let isInside = (ref: React.ref<Nullable.t<Dom.element>>, e) =>
          ref.current
          ->Nullable.toOption
          ->Option.mapOr(false, el => el->contains(e->target))

        let onMouseDown = e =>
          if !isInside(anchorRef, e) && !isInside(popoverRef, e) {
            close()
          }

        updatePlacement()
        // Capture, to also follow the table's own scroll container.
        addWindowListener("scroll", updatePlacement, true)
        addWindowListener("resize", updatePlacement, false)
        addWindowListener("keydown", onKeyDown, false)
        addWindowListener("mousedown", onMouseDown, false)

        Some(
          () => {
            removeWindowListener("scroll", updatePlacement, true)
            removeWindowListener("resize", updatePlacement, false)
            removeWindowListener("keydown", onKeyDown, false)
            removeWindowListener("mousedown", onMouseDown, false)
          },
        )
      } else {
        None
      }
    , [isOpen])

    let onClick = _ =>
      if pinned {
        close()
      } else {
        cancelTimer()
        setPinned(_ => true)
      }

    <span
      className={styles["missedMonths"]}
      onMouseEnter={_ => after(openDelay, () => setHovered(_ => true))}
      onMouseLeave={_ => after(closeDelay, () => setHovered(_ => false))}
    >
      <button
        ref={ReactDOM.Ref.domRef(anchorRef)}
        className={styles["trigger"]}
        ariaLabel={Int.toString(missedMonths) ++ " missed months, show payment history"}
        ariaExpanded=isOpen
        onClick
      >
        {React.string(Int.toString(missedMonths))}
      </button>
      {switch placement {
      | None => React.null
      | Some(style) =>
        <div
          ref={ReactDOM.Ref.domRef(popoverRef)}
          className={styles["popover"]}
          style={isOpen ? style : hidden}
          role="dialog"
          ariaLabel="Payment history"
        >
          <header className={styles["popoverHeader"]}>
            <span>
              {React.string("Payment history of ")}
              {MemberSummaryTable.viewPaddedNumber(member.memberNumber, ())}
            </span>
            <button className={styles["closeBtn"]} title="Close" onClick={_ => close()}>
              <Icons.Close />
            </button>
          </header>
          <HistoryPreview api bankApi member />
        </div>
      }}
    </span>
  }
}

/* Order of the missing dues list, switched by clicking the Missed Months or
 Last Company header. One direction each, no toggling. */
type sortBy =
  | ByMissedMonths
  | ByLastCompany

module NeverPaidFilter = {
  @react.component
  let make = (~checked, ~onChange: unit => unit) =>
    <Form.CheckboxButton checked onChange={_ => onChange()}>
      {React.string("Never paid only")}
    </Form.CheckboxButton>
}

module HideSingleMissingFilter = {
  @react.component
  let make = (~checked, ~onChange: unit => unit) =>
    <Form.CheckboxButton checked onChange={_ => onChange()}>
      {React.string("Hide single missing payment")}
    </Form.CheckboxButton>
}

/* Dues notes on the missing-dues pages, shared with MyWorkplaceMissingDues:
   the Notes column, the drawer, and the Dues Notes section for members whose
   thread outlived their missing dues. */
module DuesNotes = {
  let column = (
    byId: Map.t<string, NoteData.summary>,
    onOpen: MemberData.summary => unit,
  ): MemberSummaryTable.column => Custom({
    name: "Notes",
    minMax: ("250px", "4fr"),
    view: r => <Notes.Cell summary={Notes.summaryOf(byId, r.id)} onOpen={() => onOpen(r)} />,
  })

  /* Members with a dues thread who are not in the bank API's missing list for
     the selected period. Empty until the missing list has loaded, so nobody
     shows up here just because the bank data is still on its way. */
  let resolved = (
    ~byId: Map.t<string, NoteData.summary>,
    ~missing: Api.webData<'a>,
    ~isMissing: MemberData.summary => bool,
    members: Api.webData<array<MemberData.summary>>,
  ) =>
    switch (missing, members) {
    | (Success(_), Success(rows)) =>
      rows->Array.filter(m => Notes.summaryOf(byId, m.id)->Option.isSome && !isMissing(m))
    | _ => []
    }

  module Drawer = {
    @react.component
    let make = (
      ~api: Api.t,
      ~member: option<MemberData.summary>,
      ~onClose: unit => unit,
      ~onChange: unit => unit,
    ) =>
      switch member {
      | None => React.null
      | Some(m) => {
          let name = [m.firstName, m.lastName]->Array.filterMap(n => n)->Array.joinUnsafe(" ")
          <Notes.Drawer
            title={<>
              {React.string("Dues notes of ")}
              {MemberSummaryTable.viewPaddedNumber(m.memberNumber, ())}
              {React.string(" " ++ name)}
            </>}
            onClose
          >
            <Notes.Panel
              key={Uuid.toString(m.id)} api target=NoteData.MemberDues targetId=m.id onChange
            />
          </Notes.Drawer>
        }
      }
  }

  module Section = {
    @react.component
    let make = (~members: array<MemberData.summary>, ~notesColumn: MemberSummaryTable.column) =>
      if Array.length(members) == 0 {
        React.null
      } else {
        <>
          <h2> {React.string("Dues Notes")} </h2>
          <p>
            {React.string(
              "Members with dues notes who are not missing any dues in the selected period. Read the outcome and clear the notes.",
            )}
          </p>
          <MemberSummaryTable
            data=RemoteData.Success(members)
            columns=[Id, MemberNumber, FirstName, LastName, notesColumn, Email, Phone]
          />
        </>
      }
  }
}

/* Split out so a change of year or month remounts it (via `key` in the parent)
 and re-runs the bank API request -- Hook.getData only fetches on mount. */
module MissingList = {
  @react.component
  let make = (
    ~api: Api.t,
    ~bankApi: Api.t,
    ~year: int,
    ~month: option<int>,
    ~members: Api.webData<array<MemberData.summary>>,
    ~onlyNeverPaid: bool,
    ~hideSingleMissing: bool,
    ~sortBy: sortBy,
    ~setSortBy: sortBy => unit,
  ) => {
    let path = switch month {
    | Some(m) => "/payments/" ++ Int.toString(year) ++ "/" ++ Int.toString(m) ++ "/missing"
    | None => "/payments/" ++ Int.toString(year) ++ "/missing"
    }

    let (missing, _, _) = bankApi->Hook.getData(~path, ~decoder=PaymentData.Decode.missing)

    let missingByNumber = missing->RemoteData.unwrap(~default=Map.make(), rows =>
      rows
      ->Array.map((r: PaymentData.missingPaymentMember) => (
        r.memberNumber,
        (r.totalMissedMonths, r.hasEverPaid),
      ))
      ->Map.fromArray
    )

    let missedOf = (m: MemberData.summary) => {
      let (months, _) = missingByNumber->Map.get(m.memberNumber)->Option.getOr((0, true))
      months
    }

    let hasEverPaidOf = (m: MemberData.summary) => {
      let (_, everPaid) = missingByNumber->Map.get(m.memberNumber)->Option.getOr((0, true))
      everPaid
    }

    let byMissedMonths = (a, b) => missedOf(b) - missedOf(a)

    // A to Z, members with no company last; most missed months first within one.
    let byLastCompany = (a, b) =>
      switch (MemberSummaryTable.lastCompanyOf(a), MemberSummaryTable.lastCompanyOf(b)) {
      | (Some(x), Some(y)) =>
        switch String.localeCompare(x, y) {
        | 0.0 => byMissedMonths(a, b)
        | order => order > 0.0 ? 1 : -1
        }
      | (Some(_), None) => -1
      | (None, Some(_)) => 1
      | (None, None) => byMissedMonths(a, b)
      }

    let sortHeader = (by): DataTable.sort => {active: sortBy == by, onSelect: () => setSortBy(by)}

    let (notesById, _, refreshNotes) = Notes.useSummary(api, NoteData.MemberDues)
    let (notesFor, setNotesFor) = React.useState(() => None)
    let notesColumn = DuesNotes.column(notesById, m => setNotesFor(_ => Some(m)))

    let displayedMembers = members->RemoteData.map(rows =>
      rows
      ->Array.filter(m =>
        missingByNumber->Map.has(m.memberNumber) &&
        (!onlyNeverPaid || !hasEverPaidOf(m)) &&
        (!hideSingleMissing || missedOf(m) != 1)
      )
      ->Array.toSorted((a, b) =>
        (
          switch sortBy {
          | ByMissedMonths => byMissedMonths
          | ByLastCompany => byLastCompany
          }
        )(a, b)->Int.toFloat
      )
    )

    <>
      {switch missing {
      | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | _ => React.null
      }}
      <MemberSummaryTable
        data=displayedMembers
        columns=[
          Id,
          MemberNumber,
          FirstName,
          LastName,
          Custom({
            name: "Missed Months",
            minMax: ("140px", "1fr"),
            view: r => <MissedMonthsCell api bankApi member=r missedMonths={missedOf(r)} />,
            sort: sortHeader(ByMissedMonths),
          }),
          Custom({
            name: "Ever Paid",
            minMax: ("120px", "1fr"),
            view: r => React.string(hasEverPaidOf(r) ? "Yes" : "No"),
          }),
          notesColumn,
          LastCompany,
          Email,
          Phone,
        ]
        sortOf={c =>
          switch c {
          | LastCompany => Some(sortHeader(ByLastCompany))
          | _ => None
          }}
      >
        <p> {React.string("No members are missing dues for the selected period.")} </p>
      </MemberSummaryTable>
      <DuesNotes.Section
        members={members->DuesNotes.resolved(~byId=notesById, ~missing, ~isMissing=m =>
          missingByNumber->Map.has(m.memberNumber)
        )}
        notesColumn
      />
      <DuesNotes.Drawer
        api member=notesFor onClose={() => setNotesFor(_ => None)} onChange=refreshNotes
      />
    </>
  }
}

/* Split out for the same reason as MissingList -- remount on year/month
 change so Hook.getData re-fetches. */
module CommentedList = {
  @react.component
  let make = (
    ~bankApi: Api.t,
    ~year: int,
    ~month: option<int>,
    ~members: Api.webData<array<MemberData.summary>>,
  ) => {
    let path = switch month {
    | Some(m) => "/payments/" ++ Int.toString(year) ++ "/" ++ Int.toString(m) ++ "/commented"
    | None => "/payments/" ++ Int.toString(year) ++ "/commented"
    }

    let (commented, _, _) = bankApi->Hook.getData(~path, ~decoder=PaymentData.Decode.commented)

    let commentByNumber = commented->RemoteData.unwrap(~default=Map.make(), rows =>
      rows
      ->Array.map((r: PaymentData.commentedTransaction) => (r.memberNumber, r.adminComment))
      ->Map.fromArray
    )

    let commentOf = (m: MemberData.summary) =>
      commentByNumber->Map.get(m.memberNumber)->Option.getOr("")

    let displayedMembers =
      members->RemoteData.map(rows =>
        rows->Array.filter(m => commentByNumber->Map.has(m.memberNumber))
      )

    // Nothing to show and nothing pending -- don't render an empty table.
    let isEmpty = switch displayedMembers {
    | Success([]) => true
    | _ => false
    }

    if isEmpty {
      React.null
    } else {
      <>
        <h2> {React.string("Commented Transactions")} </h2>
        {switch commented {
        | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        | _ => React.null
        }}
        <MemberSummaryTable
          data=displayedMembers
          columns=[
            Id,
            MemberNumber,
            FirstName,
            LastName,
            Custom({
              name: "Admin Comment",
              minMax: ("250px", "3fr"),
              view: r => React.string(commentOf(r)),
            }),
            LastCompany,
            Email,
            Phone,
          ]
        >
          <p> {React.string("No commented transactions for the selected period.")} </p>
        </MemberSummaryTable>
      </>
    }
  }
}

/* Split out so a change of tab remounts it (via `key` in the parent) and
 re-fetches the member list for that status. */
module Body = {
  @react.component
  let make = (
    ~api: Api.t,
    ~bankApi: Api.t,
    ~tab: option<MemberData.status>,
    ~year: int,
    ~month: option<int>,
    ~onlyNeverPaid: bool,
    ~hideSingleMissing: bool,
    ~sortBy: sortBy,
    ~setSortBy: sortBy => unit,
  ) => {
    let membersPath = switch tab {
    | None => "/members"
    | Some(CurrentMember) => "/members/current"
    | Some(PastMember) => "/members/past"
    | Some(NewMember) => "/members/new"
    }

    let (members, _, _) =
      api->Hook.getData(~path=membersPath, ~decoder=Json.Decode.array(MemberData.Decode.summary))

    let monthKey = month->Option.mapOr("all", x => Int.toString(x))

    <>
      <MissingList
        key={Int.toString(year) ++ "-" ++ monthKey}
        api
        bankApi
        year
        month
        members
        onlyNeverPaid
        hideSingleMissing
        sortBy
        setSortBy
      />
      <CommentedList
        key={"commented-" ++ Int.toString(year) ++ "-" ++ monthKey} bankApi year month members
      />
    </>
  }
}

let tabKey = (tab: option<MemberData.status>) =>
  switch tab {
  | None => "all"
  | Some(CurrentMember) => "current"
  | Some(PastMember) => "past"
  | Some(NewMember) => "new"
  }

@react.component
let make = (~api: Api.t, ~bankApi: Api.t, ~bankEnabled: bool) => {
  let currentYear = Date.make()->Date.getFullYear

  let url = RescriptReactRouter.useUrl()
  let (year, month) = YearMonthFilter.fromSearch(~search=url.search, ~currentYear)

  let (onlyNeverPaid, setOnlyNeverPaid) = React.useState(_ => false)
  let (hideSingleMissing, setHideSingleMissing) = React.useState(_ => false)
  let (sortBy, setSortBy) = React.useState(_ => ByMissedMonths)

  let tabHandlers = Tabbed.make(Some(MemberData.CurrentMember))
  let (activeTab, _) = tabHandlers

  <Page requireAnyRole=[ListMembers]>
    <Page.Title>
      {React.string("Missing Dues (data synced once per day every morning)")}
    </Page.Title>
    <RequireFeature enabled=bankEnabled>
      <SessionContext.RequireBankRole anyOf=[Session.PaymentHistory]>
        <YearMonthFilter basePath="/missing-dues" year month currentYear />
        <Button.Panel>
          <NeverPaidFilter checked=onlyNeverPaid onChange={() => setOnlyNeverPaid(v => !v)} />
          <HideSingleMissingFilter
            checked=hideSingleMissing onChange={() => setHideSingleMissing(v => !v)}
          />
        </Button.Panel>
        <Tabbed.Tabs>
          <Tabbed.Tab
            value=Some(MemberData.CurrentMember) handlers=tabHandlers color=Some("var(--color6)")
          >
            <span> {React.string("Current")} </span>
          </Tabbed.Tab>
          <Tabbed.Tab
            value=Some(MemberData.PastMember) handlers=tabHandlers color=Some("var(--color7)")
          >
            <span> {React.string("Past")} </span>
          </Tabbed.Tab>
          <Tabbed.TabSpacer />
          <Tabbed.Tab value=None handlers=tabHandlers color=Some("var(--color1)")>
            <span> {React.string("All")} </span>
          </Tabbed.Tab>
        </Tabbed.Tabs>
        <Body
          key={tabKey(activeTab)}
          api
          bankApi
          tab=activeTab
          year
          month
          onlyNeverPaid
          hideSingleMissing
          sortBy
          setSortBy={by => setSortBy(_ => by)}
        />
      </SessionContext.RequireBankRole>
    </RequireFeature>
  </Page>
}
