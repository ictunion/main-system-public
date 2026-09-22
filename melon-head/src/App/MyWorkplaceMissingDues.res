open Belt

module NeverPaidFilter = {
  @react.component
  let make = (~checked, ~onChange: unit => unit) =>
    <Form.CheckboxButton checked onChange={_ => onChange()}>
      {React.string("Never paid only")}
    </Form.CheckboxButton>
}

/* Rep-facing counterpart to MissingDues: same picker (YearMonthFilter) and
   table (WorkplaceMemberSummaryTable), but scoped to the caller's own
   workplace. The bank endpoints take no workplace ID -- the bank service
   resolves the caller's workplace from their Keycloak executive-committee
   group, same as /workplaces/mine -- so unlike MissingDues there is no
   union-wide member list to cross-reference and filter by hand; the roster
   already comes pre-scoped from /workplaces/mine/members. */
module MissingList = {
  @react.component
  let make = (
    ~bankApi: Api.t,
    ~api: Api.t,
    ~year: int,
    ~month: option<int>,
    ~onlyNeverPaid: bool,
  ) => {
    let path = switch month {
    | Some(m) =>
      "/payments/workplace/" ++ Int.toString(year) ++ "/" ++ Int.toString(m) ++ "/missing"
    | None => "/payments/workplace/" ++ Int.toString(year) ++ "/missing"
    }

    let (missing, _, _) = bankApi->Hook.getData(~path, ~decoder=PaymentData.Decode.missing)

    let (members, _, _) =
      api->Hook.getData(
        ~path="/workplaces/mine/members",
        ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.member),
      )

    let missingByNumber =
      missing->RemoteData.unwrap(~default=Map.Int.empty, rows =>
        rows
        ->Array.map((r: PaymentData.missingPaymentMember) => (
            r.memberNumber,
            (r.totalMissedMonths, r.hasEverPaid),
          ))
        ->Map.Int.fromArray
      )

    let missedOf = (m: WorkplaceData.mineMember) => {
      let (months, _) = missingByNumber->Map.Int.getWithDefault(m.memberNumber, (0, true))
      months
    }

    let hasEverPaidOf = (m: WorkplaceData.mineMember) => {
      let (_, everPaid) = missingByNumber->Map.Int.getWithDefault(m.memberNumber, (0, true))
      everPaid
    }

    let displayedMembers =
      members->RemoteData.map(rows =>
        rows
        ->Array.keep(m =>
          missingByNumber->Map.Int.has(m.memberNumber) && (!onlyNeverPaid || !hasEverPaidOf(m))
        )
        ->Array.copy
        ->Js.Array2.sortInPlaceWith((a, b) => missedOf(b) - missedOf(a))
      )

    <>
      {switch missing {
      | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | _ => React.null
      }}
      <WorkplaceMemberSummaryTable
        data=displayedMembers
        extraColumns=[
          {
            name: "Missed Months",
            minMax: ("140px", "1fr"),
            view: r => React.string(Int.toString(missedOf(r))),
          },
          {
            name: "Ever Paid",
            minMax: ("120px", "1fr"),
            view: r => React.string(hasEverPaidOf(r) ? "Yes" : "No"),
          },
        ]>
        <p> {React.string("No members are missing dues for the selected period.")} </p>
      </WorkplaceMemberSummaryTable>
    </>
  }
}

@react.component
let make = (~api: Api.t, ~bankApi: Api.t) => {
  let currentYear = Js.Date.make()->Js.Date.getFullYear->Float.toInt

  let url = RescriptReactRouter.useUrl()
  let (year, month) = YearMonthFilter.fromSearch(~search=url.search, ~currentYear)

  let (onlyNeverPaid, setOnlyNeverPaid) = React.useState(_ => false)

  <Page requireAnyRole=[ListOwnWorkplaceMembers]>
    <Page.Title>
      {React.string("Missing Dues (data synced once per day every morning)")}
    </Page.Title>
    <YearMonthFilter basePath="/my-workplace-missing-dues" year month currentYear />
    <Button.Panel>
      <NeverPaidFilter checked=onlyNeverPaid onChange={() => setOnlyNeverPaid(v => !v)} />
    </Button.Panel>
    <MissingList
      key={Int.toString(year) ++ "-" ++ month->Option.mapWithDefault("all", Int.toString)}
      bankApi
      api
      year
      month
      onlyNeverPaid
    />
  </Page>
}
