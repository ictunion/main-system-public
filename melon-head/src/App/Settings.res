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
  let make = (
    ~label: string,
    ~count: option<int>,
    ~onFixAll: unit => unit,
    ~onNavigate: unit => unit,
    ~error: option<Api.error>=?,
  ) => {
    <div className={styles["verificationRow"]}>
      <span className={styles["verificationRowLabel"]}> {React.string(label)} </span>
      {switch error {
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      | None =>
        switch count {
        | None => <span className={styles["verificationRowCell"]}> {React.string("…")} </span>
        | Some(0) =>
          <span className={styles["verificationAllGood"]}> {React.string("✓ All good")} </span>
        | Some(n) =>
          <>
            <span className={styles["verificationRowCell"]}>
              {React.string(n->Belt.Int.toString)}
            </span>
            <span className={styles["verificationRowActions"]}>
              <Button variant=Button.Cta onClick={_ => onFixAll()}>
                {React.string("Fix all")}
              </Button>
              <button className={styles["verificationButton"]} onClick={_ => onNavigate()}>
                {React.string("→")}
              </button>
            </span>
          </>
        }
      }}
    </div>
  }
}

type tabs =
  | Permissions
  | DataVerification

type detailView =
  | Overview
  | MissingSubView
  | MissingGroupView
  | MissingWorkplaceGroupView

let oidGroupMemberDecoder = Json.Decode.array(
  Json.Decode.object(field => field.required(. "id", Data.Uuid.decode)),
)

module DataVerificationContent = {
  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~membersGroupId: option<Data.Uuid.t>) => {
    let (members, _, reloadMembers) =
      api->Hook.getData(~path="/members/", ~decoder=Json.Decode.array(MemberData.Decode.summary))
    let (workplaces, _, _) =
      api->Hook.getData(
        ~path="/workplaces/",
        ~decoder=Json.Decode.array(WorkplaceData.Decode.summary),
      )
    let (detailView, setDetailView) = React.useState(_ => Overview)

    let (groupMembers: Api.webData<array<Data.Uuid.t>>, setGroupMembers) = React.useState(
      RemoteData.init,
    )
    let (groupMembersKey, setGroupMembersKey) = React.useState(_ => 0)
    let reloadGroupMembers = () => setGroupMembersKey(k => k + 1)

    let (
      workplaceGroupMembersMap: Api.webData<Belt.Map.String.t<Belt.Set.String.t>>,
      setWorkplaceGroupMembersMap,
    ) = React.useState(RemoteData.init)
    let (workplaceGroupKey, setWorkplaceGroupKey) = React.useState(_ => 0)
    let reloadWorkplaceGroups = () => setWorkplaceGroupKey(k => k + 1)

    React.useEffect2(() => {
      switch workplaces {
      | Success(wps) =>
        if wps->Array.length == 0 {
          setWorkplaceGroupMembersMap(_ => RemoteData.Success(Belt.Map.String.empty))
          None
        } else {
          setWorkplaceGroupMembersMap(RemoteData.setLoading)
          let remaining = ref(wps->Array.length)
          let failed = ref(false)
          let acc = ref(Belt.Map.String.empty)
          let requests = wps->Array.map((wp: WorkplaceData.summary) =>
            api->Api.getJson(
              ~path="/oidc/" ++ Data.Uuid.toString(wp.keycloakGroupId),
              ~decoder=oidGroupMemberDecoder,
            )
          )
          requests->Array.forEachWithIndex((i, req) => {
            let wp = wps->Array.getUnsafe(i)
            req->Future.get(res => {
              remaining := remaining.contents - 1
              switch res {
              | Error(e) =>
                if !failed.contents {
                  failed := true
                  setWorkplaceGroupMembersMap(_ => RemoteData.Failure(e))
                }
              | Ok(subs) =>
                let subSet = subs->Array.map(Data.Uuid.toString)->Belt.Set.String.fromArray
                acc :=
                  acc.contents->Belt.Map.String.set(Data.Uuid.toString(wp.id), subSet)
                if remaining.contents === 0 && !failed.contents {
                  setWorkplaceGroupMembersMap(_ => RemoteData.Success(acc.contents))
                }
              }
            })
          })
          Some(() => requests->Array.forEach(req => Future.cancel(req)))
        }
      | _ => None
      }
    }, (workplaces, workplaceGroupKey))

    React.useEffect2(() => {
      switch membersGroupId {
      | None => None
      | Some(groupId) =>
        setGroupMembers(RemoteData.setLoading)
        let req =
          api->Api.getJson(
            ~path="/oidc/" ++ Data.Uuid.toString(groupId),
            ~decoder=oidGroupMemberDecoder,
          )
        req->Future.get(res => setGroupMembers(_ => RemoteData.fromResult(res)))
        Some(() => Future.cancel(req))
      }
    }, (membersGroupId, groupMembersKey))

    let membersWithoutSub =
      members->RemoteData.map(ms =>
        ms->Array.keep(m => m.sub->Option.isNone && m.leftAt->Option.isNone)
      )

    let missingSub = membersWithoutSub->RemoteData.map(Array.length)

    let membersNotInGroup = switch (members, groupMembers) {
    | (Success(ms), Success(groupIds)) =>
      let groupIdSet = groupIds->Array.map(Data.Uuid.toString)->Belt.Set.String.fromArray
      RemoteData.Success(
        ms->Array.keep(m =>
          m.leftAt->Option.isNone &&
            m.sub->Option.mapWithDefault(false, sub =>
              !(groupIdSet->Belt.Set.String.has(Data.Uuid.toString(sub)))
            )
        ),
      )
    | (Failure(e), _) | (_, Failure(e)) => RemoteData.Failure(e)
    | (Loading, _) | (_, Loading) => RemoteData.Loading
    | _ => RemoteData.Idle
    }

    let missingGroup = membersNotInGroup->RemoteData.map(Array.length)

    let membersWithMissingWorkplaceGroup = switch (members, workplaces, workplaceGroupMembersMap) {
    | (Success(ms), Success(_), Success(groupMap)) =>
      RemoteData.Success(
        ms->Array.keep(m =>
          m.leftAt->Option.isNone &&
            m.workplaceIds->Array.length > 0 &&
            m.sub->Option.mapWithDefault(false, sub => {
              let subStr = Data.Uuid.toString(sub)
              m.workplaceIds
              ->Array.keep(wpId =>
                switch groupMap->Belt.Map.String.get(Data.Uuid.toString(wpId)) {
                | None => false
                | Some(subSet) => !(subSet->Belt.Set.String.has(subStr))
                }
              )
              ->Array.length > 0
            })
        ),
      )
    | (Failure(e), _, _) | (_, Failure(e), _) | (_, _, Failure(e)) => RemoteData.Failure(e)
    | (Loading, _, _) | (_, Loading, _) | (_, _, Loading) => RemoteData.Loading
    | _ => RemoteData.Idle
    }

    let missingWorkplaceGroup = membersWithMissingWorkplaceGroup->RemoteData.map(Array.length)

    let pollUntilResolved = (~maxAttempts: int) => {
      let attempt = ref(0)
      let rec poll = () => {
        let _ = Js.Global.setTimeout(() => {
          attempt := attempt.contents + 1
          let req = reloadMembers()
          req->Future.get(res => {
            let stillMissing = switch res {
            | Ok(ms) =>
              ms->Array.keep(m => m.sub->Option.isNone && m.leftAt->Option.isNone)->Array.length
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

    let doFixAllGroup = () => {
      switch (membersGroupId, membersNotInGroup) {
      | (Some(groupId), Success(ms)) =>
        let rec processNext = (members: list<MemberData.summary>) =>
          switch members {
          | list{} => reloadGroupMembers()
          | list{m, ...rest} =>
            let _ =
              api
              ->Api.put(
                ~path="/members/" ++
                Data.Uuid.toString(m.id) ++
                "/oidc_groups/" ++
                Data.Uuid.toString(groupId),
                ~decoder=Api.Decode.acceptedResponse,
              )
              ->Future.get(_ => processNext(rest))
          }
        processNext(ms->Belt.List.fromArray)
      | _ => ()
      }
    }

    let doFixAllWorkplaceGroup = () => {
      switch (membersWithMissingWorkplaceGroup, workplaces, workplaceGroupMembersMap) {
      | (Success(ms), Success(wps), Success(groupMap)) =>
        let workplaceKeycloakGroupMap = wps->Array.reduce(Belt.Map.String.empty, (acc, wp) =>
          acc->Belt.Map.String.set(Data.Uuid.toString(wp.id), wp.keycloakGroupId)
        )
        let pairs = ref(list{})
        ms->Array.forEach(m =>
          switch m.sub {
          | None => ()
          | Some(sub) =>
            let subStr = Data.Uuid.toString(sub)
            m.workplaceIds->Array.forEach(wpId => {
              let wpIdStr = Data.Uuid.toString(wpId)
              switch (
                groupMap->Belt.Map.String.get(wpIdStr),
                workplaceKeycloakGroupMap->Belt.Map.String.get(wpIdStr),
              ) {
              | (Some(subSet), Some(keycloakGroupId))
                when !(subSet->Belt.Set.String.has(subStr)) =>
                pairs := list{(m.id, keycloakGroupId), ...pairs.contents}
              | _ => ()
              }
            })
          }
        )
        let rec processNext = (pairs: list<(Data.Uuid.t, Data.Uuid.t)>) =>
          switch pairs {
          | list{} => reloadWorkplaceGroups()
          | list{(memberId, keycloakGroupId), ...rest} =>
            let _ =
              api
              ->Api.put(
                ~path="/members/" ++
                Data.Uuid.toString(memberId) ++
                "/oidc_groups/" ++
                Data.Uuid.toString(keycloakGroupId),
                ~decoder=Api.Decode.acceptedResponse,
              )
              ->Future.get(_ => processNext(rest))
          }
        processNext(pairs.contents)
      | _ => ()
      }
    }

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(
        modal,
        Members.newNoteModal(
          ~api,
          ~modal,
          ~refreshMembers=reloadMembers,
          uuid,
          ~isApplication=false,
          note,
        ),
      )

    switch detailView {
    | MissingSubView =>
      <div className={styles["missingSubView"]}>
        <div className={styles["backButton"]}>
          <Button onClick={_ => setDetailView(_ => Overview)}> {React.string("← Back")} </Button>
        </div>
        <MemberSummaryTable data=membersWithoutSub onNoteClick=openNewNoteModal>
          {React.null}
        </MemberSummaryTable>
      </div>
    | MissingGroupView =>
      <div className={styles["missingSubView"]}>
        <div className={styles["backButton"]}>
          <Button onClick={_ => setDetailView(_ => Overview)}> {React.string("← Back")} </Button>
        </div>
        <MemberSummaryTable data=membersNotInGroup onNoteClick=openNewNoteModal>
          {React.null}
        </MemberSummaryTable>
      </div>
    | MissingWorkplaceGroupView =>
      <div className={styles["missingSubView"]}>
        <div className={styles["backButton"]}>
          <Button onClick={_ => setDetailView(_ => Overview)}> {React.string("← Back")} </Button>
        </div>
        <MemberSummaryTable data=membersWithMissingWorkplaceGroup onNoteClick=openNewNoteModal>
          {React.null}
        </MemberSummaryTable>
      </div>
    | Overview =>
      <div className={styles["verificationRows"]}>
        <VerificationRow
          label="Members without Keycloak connected"
          count={missingSub->RemoteData.toOption}
          onFixAll={doFixAll}
          onNavigate={() => setDetailView(_ => MissingSubView)}
          error=?{switch members {
          | Failure(err) => Some(err)
          | _ => None
          }}
        />
        <VerificationRow
          label="Members not in Keycloak members group"
          count={missingGroup->RemoteData.toOption}
          onFixAll={doFixAllGroup}
          onNavigate={() => setDetailView(_ => MissingGroupView)}
          error=?{switch groupMembers {
          | Failure(err) => Some(err)
          | _ => None
          }}
        />
        <VerificationRow
          label="Members with missing Workplace Group in Keycloak"
          count={missingWorkplaceGroup->RemoteData.toOption}
          onFixAll={doFixAllWorkplaceGroup}
          onNavigate={() => setDetailView(_ => MissingWorkplaceGroupView)}
          error=?{switch (workplaces, workplaceGroupMembersMap) {
          | (Failure(err), _) | (_, Failure(err)) => Some(err)
          | _ => None
          }}
        />
      </div>
    }
  }
}

@react.component
let make = (
  ~api: Api.t,
  ~session: Api.webData<Session.t>,
  ~modal: Modal.Interface.t,
  ~config: Config.t,
) => {
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
      <Tabbed.Tab value=Permissions handlers=tabHandlers>
        {React.string("Permissions")}
      </Tabbed.Tab>
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
        <DataVerificationContent api modal membersGroupId=config.keycloakMembersGroupId />
      </Tabbed.Content>
    </SessionContext.RequireRole>
  </Page>
}
