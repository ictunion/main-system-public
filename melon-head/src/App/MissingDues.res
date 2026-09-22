open Belt

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

/* Split out so a change of year or month remounts it (via `key` in the parent)
   and re-runs the bank API request -- Hook.getData only fetches on mount. */
module MissingList = {
  @react.component
  let make = (
    ~bankApi: Api.t,
    ~year: int,
    ~month: option<int>,
    ~members: Api.webData<array<MemberData.summary>>,
    ~onlyNeverPaid: bool,
    ~hideSingleMissing: bool,
  ) => {
    let path = switch month {
    | Some(m) => "/payments/" ++ Int.toString(year) ++ "/" ++ Int.toString(m) ++ "/missing"
    | None => "/payments/" ++ Int.toString(year) ++ "/missing"
    }

    let (missing, _, _) = bankApi->Hook.getData(~path, ~decoder=PaymentData.Decode.missing)

    let missingByNumber =
      missing->RemoteData.unwrap(~default=Map.Int.empty, rows =>
        rows
        ->Array.map((r: PaymentData.missingPaymentMember) => (
            r.memberNumber,
            (r.totalMissedMonths, r.hasEverPaid),
          ))
        ->Map.Int.fromArray
      )

    let missedOf = (m: MemberData.summary) => {
      let (months, _) = missingByNumber->Map.Int.getWithDefault(m.memberNumber, (0, true))
      months
    }

    let hasEverPaidOf = (m: MemberData.summary) => {
      let (_, everPaid) = missingByNumber->Map.Int.getWithDefault(m.memberNumber, (0, true))
      everPaid
    }

    let displayedMembers =
      members->RemoteData.map(rows =>
        rows
        ->Array.keep(m =>
          missingByNumber->Map.Int.has(m.memberNumber) &&
          (!onlyNeverPaid || !hasEverPaidOf(m)) &&
          (!hideSingleMissing || missedOf(m) != 1)
        )
        ->Array.copy
        ->Js.Array2.sortInPlaceWith((a, b) => missedOf(b) - missedOf(a))
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
            view: r => React.string(Int.toString(missedOf(r))),
          }),
          Custom({
            name: "Ever Paid",
            minMax: ("120px", "1fr"),
            view: r => React.string(hasEverPaidOf(r) ? "Yes" : "No"),
          }),
          LastCompany,
          Email,
          Phone,
          CreatedOn,
        ]>
        <p> {React.string("No members are missing dues for the selected period.")} </p>
      </MemberSummaryTable>
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

    let commentByNumber =
      commented->RemoteData.unwrap(~default=Map.Int.empty, rows =>
        rows
        ->Array.map((r: PaymentData.commentedTransaction) => (r.memberNumber, r.adminComment))
        ->Map.Int.fromArray
      )

    let commentOf = (m: MemberData.summary) =>
      commentByNumber->Map.Int.getWithDefault(m.memberNumber, "")

    let displayedMembers =
      members->RemoteData.map(rows =>
        rows->Array.keep(m => commentByNumber->Map.Int.has(m.memberNumber))
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
            CreatedOn,
          ]>
          <p> {React.string("No commented transactions for the selected period.")} </p>
        </MemberSummaryTable>
      </>
    }
  }
}

@react.component
let make = (~api: Api.t, ~bankApi: Api.t) => {
  let currentYear = Js.Date.make()->Js.Date.getFullYear->Float.toInt

  let url = RescriptReactRouter.useUrl()
  let (year, month) = YearMonthFilter.fromSearch(~search=url.search, ~currentYear)

  let (onlyNeverPaid, setOnlyNeverPaid) = React.useState(_ => false)
  let (hideSingleMissing, setHideSingleMissing) = React.useState(_ => false)

  let (members, _, _) =
    api->Hook.getData(~path="/members", ~decoder=Json.Decode.array(MemberData.Decode.summary))

  <Page requireAnyRole=[ListMembers]>
    <Page.Title>
      {React.string("Missing Dues (data synced once per day every morning)")}
    </Page.Title>
    <SessionContext.RequireBankRole anyOf=[Session.PaymentHistory]>
      <YearMonthFilter basePath="/missing-dues" year month currentYear />
      <Button.Panel>
        <NeverPaidFilter checked=onlyNeverPaid onChange={() => setOnlyNeverPaid(v => !v)} />
        <HideSingleMissingFilter
          checked=hideSingleMissing onChange={() => setHideSingleMissing(v => !v)}
        />
      </Button.Panel>
      <MissingList
        key={Int.toString(year) ++ "-" ++ month->Option.mapWithDefault("all", Int.toString)}
        bankApi
        year
        month
        members
        onlyNeverPaid
        hideSingleMissing
      />
      <CommentedList
        key={"commented-" ++
        Int.toString(year) ++ "-" ++ month->Option.mapWithDefault("all", Int.toString)}
        bankApi
        year
        month
        members
      />
    </SessionContext.RequireBankRole>
  </Page>
}
