@module external styles: {..} = "./PaymentHistory/styles.module.scss"

open Belt

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

@get external scrollWidth: Dom.element => int = "scrollWidth"
@set external setScrollLeft: (Dom.element, int) => unit = "scrollLeft"

// Dues are owed from the month onboarding finished, or creation if it never did.
let joinDateOf = (detail: MemberData.detail) =>
  detail.onboardingFinishAt->Option.getWithDefault(detail.createdAt)

let yearOf = (date: Js.Date.t): int => date->Js.Date.getFullYear->Float.toInt
let monthOf = (date: Js.Date.t): int => date->Js.Date.getMonth->Float.toInt + 1

let findTransactions = (transactions: array<PaymentData.transaction>, ~year, ~month) =>
  transactions->Array.keep(t =>
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

// Decimal places written in an amount string ("150.50" -> 2).
let decimalsOf = (amount: string) =>
  switch amount->Js.String2.indexOf(".") {
  | -1 => 0
  | i => amount->Js.String2.length - i - 1
  }

/* Several transactions can land on the same month, so they are summed per
   currency (50 + 150 + 100 + 40 shows as 340), keeping the precision the
   bank sent. If any amount isn't a plain number, the raw amounts are listed
   instead of guessing. */
let viewAmounts = (ts: array<PaymentData.transaction>) =>
  switch ts {
  | [t] => viewAmount(t)
  | _ if ts->Array.some(t => t.amount->Float.fromString->Option.isNone) =>
    ts->Array.map(viewAmount)->Js.Array2.joinWith(" + ")
  | _ =>
    ts
    ->Array.reduce([], (acc, t) =>
      acc->Js.Array2.includes(t.currency) ? acc : acc->Array.concat([t.currency])
    )
    ->Array.map(currency => {
      let inCurrency = ts->Array.keep(t => t.currency == currency)
      let total =
        inCurrency->Array.reduce(0.0, (sum, t) =>
          sum +. t.amount->Float.fromString->Option.getWithDefault(0.0)
        )
      let decimals = inCurrency->Array.reduce(0, (d, t) => max(d, decimalsOf(t.amount)))
      total->Js.Float.toFixedWithPrecision(~digits=decimals) ++ " " ++ currency
    })
    ->Js.Array2.joinWith(" + ")
  }

/* A member owes dues starting the month they joined, and a given month's
   due is paid the month after it (a payment landing in September covers
   August), with that whole following month as the deadline -- September is
   payable until the end of October. So the previous calendar month (Due),
   the current one, and anything after it (Future) are not yet missing. A real
   transaction match always wins over both of those, though, in case a
   member paid ahead. */
type cellStatus =
  | NotMember
  | Future
  | Due
  | Paid(string)
  | Waived
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
  ~leftYearMonth: option<(int, int)>,
  ~paymentsData: Api.webData<array<PaymentData.transaction>>,
) => {
  let matches = switch paymentsData {
  | Success(transactions) => findTransactions(transactions, ~year, ~month)
  | Idle
  | Loading
  | Failure(_) => []
  }

  /* Transactions whose amount is shown on this month; the rest of the
     matches cover it but show their amount on a later month. */
  let (waivedHere, paidHere) =
    matches
    ->Array.keep(t =>
      switch mostRecentCoveredMonth(t) {
      | Some(recent) => recent.year == year && recent.month == month
      | None => true
      }
    )
    ->Array.partition(t => t.amount == "Waived")

  // Oldest month still inside its payment window: the previous calendar month.
  let (graceYear, graceMonth) =
    currentMonth == 1 ? (currentYear - 1, 12) : (currentYear, currentMonth - 1)

  switch (paidHere, waivedHere, matches->Array.get(0)->Option.flatMap(mostRecentCoveredMonth)) {
  | ([], [], Some(recent)) => PaidElsewhere(recent)
  | ([], _, _) if waivedHere->Array.length > 0 => Waived
  | (_, _, _) if paidHere->Array.length > 0 => Paid(viewAmounts(paidHere))
  | _ =>
    let afterLeaving = switch leftYearMonth {
    | Some((leftYear, leftMonth)) => year > leftYear || (year == leftYear && month >= leftMonth)
    | None => false
    }

    if year < joinYear || (year == joinYear && month < joinMonth) || afterLeaving {
      NotMember
    } else if year > graceYear || (year == graceYear && month >= graceMonth) {
      year == graceYear && month == graceMonth ? Due : Future
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
  | Due => <span className={styles["notDue"]}> {React.string("DUE")} </span>
  | Paid(amount) => <span className={styles["paid"]}> {React.string(amount)} </span>
  | Waived => <span className={styles["notDue"]}> {React.string("Waived")} </span>
  | PaidElsewhere(recent) =>
    let monthName = monthNames->Array.get(recent.month - 1)->Option.getWithDefault("")
    <span className={styles["paid"]}>
      {React.string("Paid in " ++ monthName ++ " " ++ Js.Int.toString(recent.year))}
    </span>
  | Missing => <span className={styles["missing"]}> {React.string("MISSING")} </span>
  | LoadingCell => <span> {React.string("...")} </span>
  }

/* True when any month since joining reads as Missing in the payment history.
   Only a loaded history counts: while loading, or after a failure, there is
   nothing to say the member owes anything. */
let hasMissingMonth = (
  ~joinDate: Js.Date.t,
  ~leftAt: option<Js.Date.t>,
  ~paymentsData: Api.webData<array<PaymentData.transaction>>,
) =>
  switch paymentsData {
  | Success(_) =>
    let joinYear = yearOf(joinDate)
    let joinMonth = monthOf(joinDate)
    let now = Js.Date.make()
    let currentYear = now->yearOf
    let currentMonth = now->monthOf
    let leftYearMonth = leftAt->Option.map(d => (yearOf(d), monthOf(d)))

    Array.makeBy(max(currentYear - joinYear + 1, 1), i => joinYear + i)->Array.some(year =>
      Array.makeBy(12, i => i + 1)->Array.some(month =>
        switch cellStatus(
          ~year,
          ~month,
          ~joinYear,
          ~joinMonth,
          ~currentYear,
          ~currentMonth,
          ~leftYearMonth,
          ~paymentsData,
        ) {
        | Missing => true
        | _ => false
        }
      )
    )
  | Idle
  | Loading
  | Failure(_) => false
  }

/* Month-by-month dues grid for one member, from the month they joined
   (`joinDate`) to now. Months from `leftAt` on read as NOT MEMBER. */
@react.component
let make = (
  ~paymentsData: Api.webData<array<PaymentData.transaction>>,
  ~joinDate: Js.Date.t,
  ~leftAt: option<Js.Date.t>,
) => {
  let joinYear = yearOf(joinDate)
  let joinMonth = monthOf(joinDate)
  let now = Js.Date.make()
  let currentYear = now->yearOf
  let currentMonth = now->monthOf
  let leftYearMonth = leftAt->Option.map(d => (yearOf(d), monthOf(d)))

  let years = Array.makeBy(max(currentYear - joinYear + 1, 1), i => joinYear + i)

  let statusOf = (~year, ~month) =>
    cellStatus(
      ~year,
      ~month,
      ~joinYear,
      ~joinMonth,
      ~currentYear,
      ~currentMonth,
      ~leftYearMonth,
      ~paymentsData,
    )

  let yearHandlers = Tabbed.make(currentYear)

  /* The wide grid lists years oldest to newest, so keep it scrolled to the
     right edge where the current year is. Re-run when the data arrives,
     since filled-in cells can widen the columns. */
  let wideRef = React.useRef(Js.Nullable.null)
  React.useEffect1(() => {
    wideRef.current->Js.Nullable.toOption->Option.forEach(el => el->setScrollLeft(el->scrollWidth))
    None
  }, [paymentsData])

  <div className={styles["payments"]}>
    {switch paymentsData {
    | Failure(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
    | _ => React.null
    }}
    /* Narrow screens: one year at a time, picked by tab. */
    <div className={styles["paymentsNarrow"]}>
      <Tabbed.Tabs>
        {years
        ->Array.map(year =>
          <Tabbed.Tab key={year->Js.Int.toString} value=year handlers=yearHandlers>
            {React.string(year->Js.Int.toString)}
          </Tabbed.Tab>
        )
        ->React.array}
      </Tabbed.Tabs>
      {years
      ->Array.map(year =>
        <Tabbed.Content key={year->Js.Int.toString} tab=year handlers=yearHandlers>
          <table className={styles["paymentsTable"]}>
            <tbody>
              {monthNames
              ->Array.mapWithIndex((index, name) =>
                <tr key={index->Js.Int.toString}>
                  <td> {React.string(name)} </td>
                  <td> {viewCellStatus(statusOf(~year, ~month=index + 1))} </td>
                </tr>
              )
              ->React.array}
            </tbody>
          </table>
        </Tabbed.Content>
      )
      ->React.array}
    </div>
    /* Wide screens: every year side by side, scrolling horizontally. */
    <div className={styles["paymentsWide"]} ref={ReactDOM.Ref.domRef(wideRef)}>
      <table className={styles["paymentsGrid"]}>
        <thead>
          <tr>
            <th />
            {years
            ->Array.map(year =>
              <th key={year->Js.Int.toString}> {React.string(year->Js.Int.toString)} </th>
            )
            ->React.array}
          </tr>
        </thead>
        <tbody>
          {monthNames
          ->Array.mapWithIndex((index, name) =>
            <tr key={index->Js.Int.toString}>
              <td> {React.string(name)} </td>
              {years
              ->Array.map(year =>
                <td key={year->Js.Int.toString}>
                  {viewCellStatus(statusOf(~year, ~month=index + 1))}
                </td>
              )
              ->React.array}
            </tr>
          )
          ->React.array}
        </tbody>
      </table>
    </div>
  </div>
}
