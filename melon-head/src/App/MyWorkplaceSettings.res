@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  let (detail, setDetail, _) =
    api->Hook.getData(
      ~path="/workplaces/mine",
      ~decoder=WorkplaceData.Decode.summary,
    )

  <WorkplaceDetail.View api modal detail setDetail />
}
