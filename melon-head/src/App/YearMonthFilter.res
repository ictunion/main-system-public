@module external styles: {..} = "./YearMonthFilter/styles.module.scss"

/* Year/month picker shared by MissingDues and MyWorkplaceMissingDues. Reads
   and writes its selection through the URL (`?year=&month=`) rather than
   local state, so a filtered view can be shared by link and survives
   back/forward navigation. `month=all` means every month. */

let firstYear = 2020

let monthNames = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
]

let paramValue = (search: string, key: string): option<string> =>
  search
  ->String.split("&")
  ->Array.find(pair => pair->String.startsWith(key ++ "="))
  ->Option.map(pair => pair->String.slice(~start=String.length(key) + 1))

/* Parses `RescriptReactRouter.useUrl().search`. The year falls back to the
   current year when absent or unparseable; the month defaults to "all" (not
   the current month) when absent or unparseable. */
let fromSearch = (~search: string, ~currentYear: int): (int, option<int>) => {
  let year =
    search
    ->paramValue("year")
    ->Option.flatMap(s => Int.fromString(s))
    ->Option.filter(y => y >= firstYear && y <= currentYear)
    ->Option.getOr(currentYear)

  let month = switch search->paramValue("month") {
  | None => None
  | Some("all") => None
  | Some(v) =>
    switch Int.fromString(v) {
    | Some(m) if m >= 1 && m <= 12 => Some(m)
    | _ => None
    }
  }

  (year, month)
}

@react.component
let make = (~basePath: string, ~year: int, ~month: option<int>, ~currentYear: int) => {
  let years = Array.fromInitializer(~length=currentYear - firstYear + 1, i => firstYear + i)
  let months = Array.fromInitializer(~length=12, i => i + 1)

  let pushUrl = (~year: int, ~month: option<int>) =>
    RescriptReactRouter.push(
      basePath ++
      "?year=" ++
      Int.toString(year) ++
      "&month=" ++
      month->Option.mapOr("all", x => Int.toString(x)),
    )

  let onYear = (e: JsxEvent.Form.t) => {
    let v = ReactEvent.Form.currentTarget(e)["value"]
    pushUrl(~year=v->Int.fromString->Option.getOr(currentYear), ~month)
  }

  let onMonth = (e: JsxEvent.Form.t) => {
    let v = ReactEvent.Form.currentTarget(e)["value"]
    pushUrl(~year, ~month=v == "all" ? None : v->Int.fromString)
  }

  <div className={styles["filters"]}>
    <label className={styles["filter"]}>
      {React.string("Year")}
      <select value={Int.toString(year)} onChange=onYear>
        {years
        ->Array.map(y =>
          <option key={Int.toString(y)} value={Int.toString(y)}>
            {React.string(Int.toString(y))}
          </option>
        )
        ->React.array}
      </select>
    </label>
    <label className={styles["filter"]}>
      {React.string("Month")}
      <select value={month->Option.mapOr("all", x => Int.toString(x))} onChange=onMonth>
        <option value="all"> {React.string("All months")} </option>
        {months
        ->Array.map(m =>
          <option key={Int.toString(m)} value={Int.toString(m)}>
            {React.string(monthNames->Array.get(m - 1)->Option.getOrThrow)}
          </option>
        )
        ->React.array}
      </select>
    </label>
  </div>
}
