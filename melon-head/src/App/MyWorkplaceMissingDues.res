open Belt

module NeverPaidFilter = {
  @react.component
  let make = (~checked, ~onChange: unit => unit) =>
    <Form.CheckboxButton checked onChange={_ => onChange()}>
      {React.string("Never paid only")}
    </Form.CheckboxButton>
}

/* Rep-facing counterpart to MissingDues: same picker (YearMonthFilter) and
   table (MemberSummaryTable), but scoped to the caller's own workplace. The
   bank endpoints take no workplace ID -- the bank service resolves the
   caller's workplace from their Keycloak executive-committee group, same as
   /workplaces/mine -- so unlike MissingDues there is no union-wide member
   list to cross-reference and filter by hand; the roster already comes
   pre-scoped from /workplaces/mine/members. */

/* MemberSummaryTable takes MemberData.summary; the /workplaces/mine/members
   projection carries only a subset, so the rest stays empty. Only columns
   backed by real fields are shown below. */
let toSummary = (m: WorkplaceData.mineMember): MemberData.summary => {
  id: m.id,
  memberNumber: m.memberNumber,
  firstName: m.firstName,
  lastName: m.lastName,
  email: m.email,
  phoneNumber: m.phoneNumber,
  note: None,
  city: None,
  language: None,
  leftAt: None,
  companyNames: [],
  createdAt: m.createdAt,
  workplaceIds: [m.workplaceId],
  isRepresentative: Some(m.isRepresentative),
  sub: None,
}

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
        ->Array.map(toSummary)
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
            // Rows exist only if the missing list loaded, which already needs
            // view-workplace-payment-history -- the same role the history needs.
            view: r => <MissingDues.MissedMonthsCell api bankApi member=r missedMonths={missedOf(r)} />,
          }),
          Custom({
            name: "Ever Paid",
            minMax: ("120px", "1fr"),
            view: r => React.string(hasEverPaidOf(r) ? "Yes" : "No"),
          }),
          Email,
          Phone,
        ]>
        <p> {React.string("No members are missing dues for the selected period.")} </p>
      </MemberSummaryTable>
    </>
  }
}

@react.component
let make = (~api: Api.t, ~bankApi: Api.t, ~bankEnabled: bool) => {
  let currentYear = Js.Date.make()->Js.Date.getFullYear->Float.toInt

  let url = RescriptReactRouter.useUrl()
  let (year, month) = YearMonthFilter.fromSearch(~search=url.search, ~currentYear)

  let (onlyNeverPaid, setOnlyNeverPaid) = React.useState(_ => false)

  <Page requireAnyRole=[ListOwnWorkplaceMembers]>
    <Page.Title>
      {React.string("Missing Dues (data synced once per day every morning)")}
    </Page.Title>
    <RequireFeature enabled=bankEnabled>
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
    </RequireFeature>
  </Page>
}
