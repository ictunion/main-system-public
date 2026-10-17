open Data
open Belt

/* Elected office held by a member (chairperson, board member, ...). It is
   current until its `termEndsOn` has passed, then it is past. */
type office = {
  id: Uuid.t,
  memberId: Uuid.t,
  memberNumber: int,
  firstName: option<string>,
  lastName: option<string>,
  title: string,
  electedBy: string,
  electedOn: Js.Date.t,
  termEndsOn: option<Js.Date.t>,
  createdAt: Js.Date.t,
  updatedAt: Js.Date.t,
}

/* Offices the Elected Offices page offers, and the table each one is listed
   in. The API takes free text, so a new office needs only a change here:
   add the variant, and the compiler points at `titleToString` and
   `groupOf`; then add it to `allTitles`. */
type title =
  | Chairperson
  | ViceChairperson
  | BoardMember
  | Treasurer
  | AuditCommittee

type group =
  | Board
  | Finance

// Display order, both in the office picker and within each table.
let allTitles = [Chairperson, ViceChairperson, BoardMember, Treasurer, AuditCommittee]

// The text the API stores, so changing one renames existing records' office.
let titleToString = (title: title) =>
  switch title {
  | Chairperson => "Chairperson"
  | ViceChairperson => "Vice-chairperson"
  | BoardMember => "Board member"
  | Treasurer => "Treasurer"
  | AuditCommittee => "Audit committee"
  }

// None for free text entered before (or outside) this list.
let titleFromString = (s: string): option<title> =>
  allTitles->Js.Array2.find(t => titleToString(t) == s)

let groupOf = (title: title) =>
  switch title {
  | Chairperson
  | ViceChairperson
  | BoardMember =>
    Board
  | Treasurer
  | AuditCommittee =>
    Finance
  }

let titles = allTitles->Array.map(titleToString)

let electedByOptions =["All members meeting", "Monthly members meeting", "Workplaces Council"]

let fullName = (office: office): string =>
  [office.firstName, office.lastName]->Array.keepMap(x => x)->Js.Array2.joinWith(" ")

/* Local midnight of the current day, comparable with decoded calendar days. */
let today = (): Js.Date.t => {
  let now = Js.Date.make()
  Js.Date.makeWithYMD(
    ~year=Js.Date.getFullYear(now),
    ~month=Js.Date.getMonth(now),
    ~date=Js.Date.getDate(now),
    (),
  )
}

/* Same day and month, `years` later. 29 February rolls over to 1 March. */
let addYears = (date: Js.Date.t, years: int): Js.Date.t =>
  Js.Date.makeWithYMD(
    ~year=Js.Date.getFullYear(date) +. Int.toFloat(years),
    ~month=Js.Date.getMonth(date),
    ~date=Js.Date.getDate(date),
    (),
  )

module Decode = {
  open Json.Decode

  let office = object(field => {
    id: field.required(. "id", Uuid.decode),
    memberId: field.required(. "member_id", Uuid.decode),
    memberNumber: field.required(. "member_number", int),
    firstName: field.required(. "first_name", option(string)),
    lastName: field.required(. "last_name", option(string)),
    title: field.required(. "title", string),
    electedBy: field.required(. "elected_by", string),
    electedOn: field.required(. "elected_on", Data.Decode.day),
    termEndsOn: field.required(. "term_ends_on", option(Data.Decode.day)),
    createdAt: field.required(. "created_at", date),
    updatedAt: field.required(. "updated_at", date),
  })
}

module Encode = {
  open Json.Encode

  let day = Data.Encode.day

  let fields = (~title, ~electedBy, ~electedOn, ~termEndsOn) => [
    ("title", string(title)),
    ("elected_by", string(electedBy)),
    ("elected_on", day(electedOn)),
    // null clears the term end on PATCH
    ("term_ends_on", option(day, termEndsOn)),
  ]

  let newOffice = (~memberId: string, ~title, ~electedBy, ~electedOn, ~termEndsOn) =>
    object(
      Array.concat(
        [("member_id", string(memberId))],
        fields(~title, ~electedBy, ~electedOn, ~termEndsOn),
      ),
    )

  /* Ending an office early: PATCH only the last day in office. */
  let termEnd = (termEndsOn: option<Js.Date.t>) => object([("term_ends_on", option(day, termEndsOn))])

  let patch = (~title, ~electedBy, ~electedOn, ~termEndsOn) =>
    object(fields(~title, ~electedBy, ~electedOn, ~termEndsOn))
}
