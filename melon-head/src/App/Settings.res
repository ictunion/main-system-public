@module external styles: {..} = "./Settings/styles.module.scss"

open Belt

module ViewBool = {
  @react.component
  let make = (~value: bool) => {value ? React.string("👍 yes") : React.string("👎 no")}
}

let statusRows: array<RowBasedTable.row<Api.status>> = [
  ("Http Status", s => React.string(s.httpStatus->Int.toString)),
  ("Http Message", s => React.string(s.httpMessage)),
  (
    "Keycloak Connected",
    ({authorizationConnected}) => {<ViewBool value={authorizationConnected} />},
  ),
  ("Database Connected", ({databaseConnected}) => {<ViewBool value={databaseConnected} />}),
  ("Proxy Support Enabled", ({proxySupportEnabled}) => {<ViewBool value={proxySupportEnabled} />}),
]

module VerificationRow = {
  @react.component
  let make = (~label: string, ~count: option<int>, ~onFixAll: unit => unit, ~onNavigate: unit => unit) => {
    <div className={styles["verificationRow"]}>
      <span className={styles["verificationRowLabel"]}> {React.string(label)} </span>
      {switch count {
      | None => <span className={styles["verificationRowCell"]}> {React.string("…")} </span>
      | Some(0) =>
        <span className={styles["verificationAllGood"]}>
          {React.string("✓ All good")}
        </span>
      | Some(n) =>
        <>
          <span className={styles["verificationRowCell"]}>
            {React.string(n->Belt.Int.toString)}
          </span>
          <span className={styles["verificationRowActions"]}>
            <Button variant=Button.Cta onClick={_ => onFixAll()}>
              {React.string("Fix all")}
            </Button>
            <button
              className={styles["verificationButton"]}
              onClick={_ => onNavigate()}>
              {React.string("→")}
            </button>
          </span>
        </>
      }}
    </div>
  }
}

type tabs =
  | Permissions
  | DataVerification

module DataVerificationContent = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
    let (members, _, reloadMembers) =
      api->Hook.getData(~path="/members/", ~decoder=Json.Decode.array(MemberData.Decode.summary))
    let (showMissingSub, setShowMissingSub) = React.useState(_ => false)

    let membersWithoutSub = members->RemoteData.map(ms =>
      ms->Array.keep(m => m.sub->Option.isNone && m.leftAt->Option.isNone)
    )

    let missingSub = membersWithoutSub->RemoteData.map(ms => ms->Array.length)

    let pollUntilResolved = (~maxAttempts: int) => {
      let attempt = ref(0)
      let rec poll = () => {
        let _ = Js.Global.setTimeout(() => {
          attempt := attempt.contents + 1
          let req = reloadMembers()
          req->Future.get(res => {
            let stillMissing = switch res {
            | Ok(ms) => ms->Array.keep(m => m.sub->Option.isNone && m.leftAt->Option.isNone)->Array.length
            | Error(_) => 0
            }

            if stillMissing > 0 && attempt.contents < maxAttempts {
              poll()
            }
          })
        }, 1000)
      }
      poll()
    }

    let doFixAll = () => {
      switch members {
      | Success(ms) =>
        let missing = ms->Array.keep(m => m.sub->Option.isNone && m.leftAt->Option.isNone)
        let remaining = ref(missing->Array.length)
        missing->Array.forEach(m => {
          let _ =
            api
            ->Api.postJson(
              ~path="/members/" ++ Data.Uuid.toString(m.id) ++ "/create_oid_account",
              ~decoder=Api.Decode.acceptedResponse,
              ~body=Js.Json.null,
            )
            ->Future.get(_ => {
              remaining := remaining.contents - 1
              if remaining.contents === 0 {
                pollUntilResolved(~maxAttempts=missing->Array.length)
              }
            })
        })
      | _ => ()
      }
    }

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        Members.newNoteModal(~api, ~modal, ~refreshMembers=reloadMembers, uuid, ~isApplication=false, note),
      )

    if showMissingSub {
      <div className={styles["missingSubView"]}>
        <div className={styles["backButton"]}>
          <Button onClick={_ => setShowMissingSub(_ => false)}>
            {React.string("← Back")}
          </Button>
        </div>
        <MemberSummaryTable data=membersWithoutSub onNoteClick={(id, note) => openNewNoteModal(id, note)}>
          {React.null}
        </MemberSummaryTable>
      </div>
    } else {
      <div className={styles["verificationRows"]}>
        <VerificationRow
          label="Members without Keycloak connected"
          count={missingSub->RemoteData.toOption}
          onFixAll={doFixAll}
          onNavigate={() => setShowMissingSub(_ => true)}
        />
      </div>
    }
  }
}

@react.component
let make = (~api: Api.t, ~session: Api.webData<Session.t>, ~modal: Modal.Interface.t) => {
  let (status, _, _) = api->Hook.getData(~path="/status", ~decoder=Api.Decode.status)
  let tabHandlers = Tabbed.make(Permissions)

  let permissionsRows = [
    (
      "Recognized as member",
      (session: Session.t) => {<ViewBool value={session.memberId->Option.isSome} />},
    ),
    (
      "List all applications",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListApplications)} />},
    ),
    (
      "View application detail",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ViewApplication)} />},
    ),
    (
      "Resolve (Approve or Reject) applications",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ResolveApplications)} />},
    ),
    (
      "List all members",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListMembers)} />},
    ),
    (
      "View member detail",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ViewMember)} />},
    ),
    (
      "Manage members",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
    (
      "Manage realm users",
      session => {<ViewBool value={Session.hasRealmRole(session, ~role=Session.ManageUsers)} />},
    ),
    (
      "List all workplaces",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ListWorkplaces)} />},
    ),
    (
      "Manage workplaces",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageWorkplaces)} />},
    ),
    (
      "Super-Powers (be careful!)",
      session => {<ViewBool value={Session.hasRole(session, ~role=Session.ManageMembers)} />},
    ),
  ]

  <Page>
    <Page.Title> {React.string("Settings")} </Page.Title>
    <Tabbed.Tabs>
      <Tabbed.Tab value=Permissions handlers=tabHandlers> {React.string("Permissions")} </Tabbed.Tab>
      <SessionContext.RequireRole anyOf=[Session.SuperPowers]>
        <Tabbed.Tab value=DataVerification handlers=tabHandlers>
          {React.string("Data Verification")}
        </Tabbed.Tab>
      </SessionContext.RequireRole>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Permissions handlers=tabHandlers>
      <div className={styles["grid"]}>
        <div className={styles["gridItem"]}>
          <h2 className={styles["itemTitle"]}> {React.string("Permissions")} </h2>
          <RowBasedTable rows=permissionsRows data=session title=Some("Your Permissions/Roles") />
        </div>
        <div className={styles["gridItem"]}>
          <h2 className={styles["itemTitle"]}> {React.string("System")} </h2>
          <RowBasedTable rows=statusRows data=status title=Some("Api Status") />
        </div>
      </div>
    </Tabbed.Content>
    <SessionContext.RequireRole anyOf=[Session.SuperPowers]>
      <Tabbed.Content tab=DataVerification handlers=tabHandlers>
        <DataVerificationContent api modal />
      </Tabbed.Content>
    </SessionContext.RequireRole>
  </Page>
}
