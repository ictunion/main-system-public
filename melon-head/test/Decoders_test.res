open Vitest

let iso = Date.toISOString

/* Every decoder against its fixture: it must decode, and must refuse the
 fixture with a required field removed or of the wrong type. */
let cases: array<(string, string, JSON.t => result<unit, string>)> = {
  let c = (fixture, key, decoder) => (
    fixture,
    key,
    json => Json.decode(json, decoder)->Result.map(_ => ()),
  )
  [
    c("member_summary", "member_number", MemberData.Decode.summary),
    c("member_detail", "member_number", MemberData.Decode.detail),
    c("member_occupation", "source", MemberData.Decode.occupation),
    c("member_office", "elected_on", OfficeData.Decode.office),
    c("application_summary", "registration_local", ApplicationData.Decode.summary),
    c("application_unverified_summary", "created_at", ApplicationData.Decode.unverifiedSummary),
    c("application_accepted_summary", "member_id", ApplicationData.Decode.acceptedSummary),
    c("application_rejected_summary", "rejected_at", ApplicationData.Decode.rejectedSummary),
    c("application_processing_summary", "confirmed_at", ApplicationData.Decode.processingSummary),
    c("application_invalid_summary", "invalidated_at", ApplicationData.Decode.invalidSummary),
    c("application_detail", "registration_local", ApplicationData.Decode.detail),
    c("application_workplace", "workplace", ApplicationData.Decode.workplace),
    c("workplace_summary", "member_count", WorkplaceData.Decode.summary),
    c("workplace_member", "is_representative", WorkplaceData.DecodeMine.member),
    c("note_detail", "author_name", NoteData.Decode.note),
    c("note_summary", "count", NoteData.Decode.summary),
    c("stats_applications", "accepted", StatsData.Applications.Decode.basic),
    c("stats_members", "sectoral", StatsData.Members.Decode.basic),
    c("stats_workplaces", "in_preparation", StatsData.Workplaces.Decode.basic),
    c("session", "token_claims", Session.Decode.session),
    c("user", "email", Session.Decode.user),
    c("file_info", "file_type", Data.Decode.file),
    c("payment_transaction", "covered_months", PaymentData.Decode.transaction),
    c("payment_missing_member", "has_ever_paid", PaymentData.Decode.missingPaymentMember),
    c("payment_commented_transaction", "admin_comment", PaymentData.Decode.commentedTransaction),
    c("payment_stats", "liable_members", PaymentData.Decode.paymentStats),
  ]
}

describe("decoders accept their fixture", () => {
  cases->Array.forEach(((fixture, _, decode)) =>
    test(fixture, t => t->expect(Fixture.load(fixture)->decode)->Expect.toEqual(Ok()))
  )
})

describe("decoders refuse a missing field", () => {
  cases->Array.forEach(((fixture, key, decode)) =>
    test(
      fixture ++ " without " ++ key,
      t =>
        t->expect(Fixture.withField(fixture, key, None)->decode->Result.isError)->Expect.toBe(true),
    )
  )
})

describe("decoders refuse a field of the wrong type", () => {
  cases->Array.forEach(((fixture, key, decode)) =>
    test(
      fixture ++ " with " ++ key ++ " as an object",
      t =>
        t
        ->expect(
          Fixture.withField(fixture, key, Some(JSON.Object(Dict.make())))->decode->Result.isError,
        )
        ->Expect.toBe(true),
    )
  )
})

describe("decoded values", () => {
  test("member detail", t => {
    let m = Fixture.decode("member_detail", MemberData.Decode.detail)
    t->expect(m.memberNumber)->Expect.toBe(42)
    t->expect(m.firstName)->Expect.toEqual(Some("Jana"))
    t->expect(m.createdAt->iso)->Expect.toBe("2026-05-01T10:15:30.123Z")
    t->expect(m.leftAt)->Expect.toEqual(None)
    t->expect(m->MemberData.getStatus)->Expect.toEqual(MemberData.CurrentMember)
  })

  test("calendar days keep the day the API sent", t => {
    let m = Fixture.decode("member_detail", MemberData.Decode.detail)
    t->expect(m.dateOfBirth->Option.map(Data.Day.toString))->Expect.toEqual(Some("1990-04-12"))
    let o = Fixture.decode("member_office", OfficeData.Decode.office)
    t->expect(o.electedOn->Data.Day.toString)->Expect.toBe("2026-03-01")
    t->expect(o.termEndsOn)->Expect.toEqual(None)
  })

  test("member summary keeps null company names", t => {
    let m = Fixture.decode("member_summary", MemberData.Decode.summary)
    t->expect(m.companyNames)->Expect.toEqual([Some("ACME s.r.o."), None])
    t->expect(m.isRepresentative)->Expect.toEqual(Some(true))
  })

  test("application detail status comes from confirmed_at", t => {
    let a = Fixture.decode("application_detail", ApplicationData.Decode.detail)
    t->expect(a.verifiedAt->Option.isSome)->Expect.toBe(true)
    t->expect(a->ApplicationData.getStatus)->Expect.toEqual(ApplicationData.Processing)
    t->expect(a.registrationIp)->Expect.toEqual(Some("127.0.0.1"))
  })

  test("workplace status", t => {
    let w = Fixture.decode("workplace_summary", WorkplaceData.Decode.summary)
    t->expect(w.newsletterId)->Expect.toEqual(Some(7))
    t->expect(w->WorkplaceData.getStatus)->Expect.toEqual(WorkplaceData.Established)
  })

  test("session roles: known, unknown and per client", t => {
    let s = Fixture.decode("session", Session.Decode.session)
    t
    ->expect(s.tokenClaims.orcaRoles)
    ->Expect.toEqual([Session.ListMembers, ManageNotes, UnknownOrcaRole("some-new-role")])
    t->expect(s.tokenClaims.bankRoles)->Expect.toEqual([Session.PaymentHistory])
    t->expect(s->Session.hasRole(~role=ListMembers))->Expect.toBe(true)
    t->expect(s->Session.hasRole(~role=ManageMembers))->Expect.toBe(false)
    t->expect(s.memberId)->Expect.toEqual(None)
  })

  test("session without an orca or bank client has no roles", t => {
    let json = Fixture.load("session")
    let claims = switch json {
    | Object(d) =>
      switch d->Dict.get("token_claims") {
      | Some(Object(c)) => c
      | _ => JsError.throwWithMessage("no token_claims")
      }
    | _ => JsError.throwWithMessage("not an object")
    }
    claims->Dict.set("resource_access", Object(Dict.make()))
    let s = Json.decode(json, Session.Decode.session)->Result.getOrThrow
    t->expect(s.tokenClaims.orcaRoles)->Expect.toEqual([])
    t->expect(s.tokenClaims.bankRoles)->Expect.toEqual([])
  })

  test("stats totals", t => {
    let a = Fixture.decode("stats_applications", StatsData.Applications.Decode.basic)
    t->expect(a->StatsData.Applications.all)->Expect.toBe(15)
    let m = Fixture.decode("stats_members", StatsData.Members.Decode.basic)
    t->expect(m->StatsData.Members.all)->Expect.toBe(14)
    t->expect(m->StatsData.Members.active)->Expect.toBe(11)
  })

  test("payments", t => {
    let tr = Fixture.decode("payment_transaction", PaymentData.Decode.transaction)
    t->expect(tr.coveredMonths)->Expect.toEqual([{year: 2026, month: 4}, {year: 2026, month: 5}])
    let s = Fixture.decode("payment_stats", PaymentData.Decode.paymentStats)
    t->expect(s.percentPaid)->Expect.toEqual(None)
  })

  test("note author may be missing", t => {
    let json = Fixture.withField("note_detail", "author_sub", Some(JSON.Null))
    let n = Json.decode(json, NoteData.Decode.note)->Result.getOrThrow
    t->expect(n.authorSub)->Expect.toEqual(None)
  })
})
