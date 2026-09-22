/* Roster of a workplace executive committee's own workplace.

   Both requests are unparameterised on purpose: the scope is resolved server
   side from the caller's Keycloak executive group membership, so there is no
   workplace ID in the URL for anyone to swap. */
@react.component
let make = (~api: Api.t) => {
  let (workplace, _, _) =
    api->Hook.getData(~path="/workplaces/mine", ~decoder=WorkplaceData.Decode.summary)

  let (members, _, _) =
    api->Hook.getData(
      ~path="/workplaces/mine/members",
      ~decoder=Json.Decode.array(WorkplaceData.DecodeMine.member),
    )

  let title = switch workplace {
  | RemoteData.Success(w) => "Members of " ++ w.name
  | _ => "My Workplace"
  }

  <Page requireAnyRole=[ListOwnWorkplaceMembers]>
    <Page.Title> {React.string(title)} </Page.Title>
    {switch workplace {
    | RemoteData.Failure(_) =>
      <p>
        {React.string(
          "You are not on the executive committee of any workplace, so there is nothing to show here.",
        )}
      </p>
    | _ =>
      <WorkplaceMemberSummaryTable data=members>
        <p> {React.string("There are no members in your workplace yet.")} </p>
      </WorkplaceMemberSummaryTable>
    }}
  </Page>
}
