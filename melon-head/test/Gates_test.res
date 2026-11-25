open Vitest
open Session

let shown = <span> {React.string("secret")} </span>

let render = (session, gate) => Render.withSession(session, gate)->Render.text

let success = (~orca=[], ~bank=[]) => RemoteData.Success(Render.session(~orca, ~bank))
let notReady: array<(string, Api.webData<Session.t>)> = [
  ("idle", Idle),
  ("loading", Loading),
  ("failed", Failure(Api.NetworkError)),
]

describe("SessionContext.RequireRole", () => {
  let gate = anyOf => <SessionContext.RequireRole anyOf> shown </SessionContext.RequireRole>

  test("shows children to a session with one of the roles", t => {
    t
    ->expect(render(success(~orca=[ManageNotes]), gate([ListMembers, ManageNotes])))
    ->Expect.toBe("secret")
  })

  test("hides children from a session without the roles", t => {
    t->expect(render(success(~orca=[ListMembers]), gate([ManageMembers])))->Expect.toBe("")
  })

  test("an unknown role never matches a known one", t => {
    t
    ->expect(render(success(~orca=[UnknownOrcaRole("manage-members")]), gate([ManageMembers])))
    ->Expect.toBe("")
  })

  test("no required roles means any logged-in session", t => {
    t->expect(render(success(), gate([])))->Expect.toBe("secret")
  })

  notReady->Array.forEach(((name, session)) =>
    test(
      "hides children while the session is " ++ name,
      t => {
        t->expect(render(session, gate([])))->Expect.toBe("")
      },
    )
  )
})

describe("SessionContext.RequireBankRole", () => {
  let gate = anyOf => <SessionContext.RequireBankRole anyOf> shown </SessionContext.RequireBankRole>

  test("shows children to a session with the bank role", t => {
    t
    ->expect(render(success(~bank=[PaymentHistory]), gate([PaymentHistory])))
    ->Expect.toBe("secret")
  })

  test("hides children from a session without it", t => {
    t
    ->expect(render(success(~bank=[WorkplacePaymentHistory]), gate([PaymentHistory])))
    ->Expect.toBe("")
  })

  test("orca roles do not count as bank roles", t => {
    t->expect(render(success(~orca=[SuperPowers]), gate([PaymentHistory])))->Expect.toBe("")
  })

  notReady->Array.forEach(((name, session)) =>
    test(
      "hides children while the session is " ++ name,
      t => {
        t->expect(render(session, gate([])))->Expect.toBe("")
      },
    )
  )
})

describe("RequireFeature", () => {
  test("shows children only when enabled", t => {
    t
    ->expect(<RequireFeature enabled=true> shown </RequireFeature>->Render.text)
    ->Expect.toBe("secret")
    t->expect(<RequireFeature enabled=false> shown </RequireFeature>->Render.text)->Expect.toBe("")
  })
})

describe("Page requireAnyRole", () => {
  let page = <Page requireAnyRole=[ListMembers]> shown </Page>

  test("renders the page for a session with the role", t => {
    t->expect(render(success(~orca=[ListMembers]), page))->Expect.String.toContain("secret")
  })

  test("renders Forbidden instead of the page otherwise", t => {
    let text = render(success(~orca=[ListOwnWorkplace]), page)
    t->expect(text)->Expect.String.toContain("Forbidden")
    t->expect(text->String.includes("secret"))->Expect.toBe(false)
  })
})

describe("WorkplaceDetail: admin vs workplace rep", () => {
  let detail = RemoteData.Success(Fixture.decode("workplace_summary", WorkplaceData.Decode.summary))
  let modal: Modal.Interface.t = {closeModal: () => (), openModal: _ => (), state: None}
  // Never called: server rendering runs no effects or handlers.
  let api: Api.t = Obj.magic()
  let view = orca =>
    render(success(~orca), <WorkplaceDetail.View api modal detail setDetail={_ => ()} />)

  let adminOnly = ["Edit", "Back to workplaces", "Files", "Generate Document", "Cancel Workplace"]

  test("admin sees the admin actions", t => {
    let text = view([ListWorkplaces, ManageWorkplaces])
    adminOnly->Array.forEach(label => t->expect(text)->Expect.String.toContain(label))
  })

  test("rep sees none of the admin actions", t => {
    let text = view([ListOwnWorkplace, ListOwnWorkplaceMembers])
    t->expect(text)->Expect.String.toContain("Show workplace members")
    adminOnly->Array.forEach(
      label => t->expect((label, text->String.includes(label)))->Expect.toEqual((label, false)),
    )
  })
})
