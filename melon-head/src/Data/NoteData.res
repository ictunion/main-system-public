open Data

/* What a note is attached to. `MemberDues` is a member's missing-dues thread,
 kept apart from the general `Member` notes because workplace reps can see it. */
type target =
  | Application
  | Member
  | MemberDues
  | Workplace

let targetToString = (target: target): string =>
  switch target {
  | Application => "application"
  | Member => "member"
  | MemberDues => "member_dues"
  | Workplace => "workplace"
  }

type note = {
  id: Uuid.t,
  body: string,
  authorSub: option<Uuid.t>,
  authorName: string,
  createdAt: Date.t,
}

/* One per target that has at least one note -- what a table cell shows. */
type summary = {
  targetId: Uuid.t,
  count: int,
  latestBody: string,
}

module Decode = {
  open Json.Decode

  let note = object(field => {
    id: field.required("id", Uuid.decode),
    body: field.required("body", string),
    authorSub: field.required("author_sub", option(Uuid.decode)),
    authorName: field.required("author_name", string),
    createdAt: field.required("created_at", date),
  })

  let summary = object(field => {
    targetId: field.required("target_id", Uuid.decode),
    count: field.required("count", int),
    latestBody: field.required("latest_body", string),
  })
}

module Encode = {
  open! Json.Encode

  let strOption = str => {
    if str == "" {
      null
    } else {
      string(str)
    }
  }

  // Legacy single-note endpoints, removed in phase 3.
  let note = (note: string) => object([("note", strOption(note))])

  let body = (body: string) => object([("body", string(body))])
}
