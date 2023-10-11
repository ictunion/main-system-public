open Belt

@react.component
let make = (~api: Api.t) => {
  let (members, _) =
    api->Hook.getData(~path="/members", ~decoder=Json.Decode.array(MemberData.Decode.summary))

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Members")} </Page.Title>
    <DataTable
      data=members
      columns=[
        {
          name: "ID",
          minMax: ("100px", "1fr"),
          view: r => <Link.Uuid uuid={r.id} toPath={uuid => "/members/" ++ uuid} />,
        },
        {
          name: "Member Number",
          minMax: ("200px", "1fr"),
          view: r => React.string(r.memberNumber->Int.toString),
        },
        {
          name: "Left On",
          minMax: ("150px", "1fr"),
          view: r =>
            React.string(r.leftAt->Option.mapWithDefault("--", Js.Date.toLocaleDateString)),
        },
        {
          name: "First Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.firstName->Option.getWithDefault("--")),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => React.string(r.lastName->Option.getWithDefault("--")),
        },
        {
          name: "Email",
          minMax: ("250px", "2fr"),
          view: r =>
            r.email->Option.mapWithDefault(React.string("--"), email => <Link.Email email />),
        },
        {
          name: "Phone",
          minMax: ("220px", "2fr"),
          view: r =>
            r.phoneNumber->Option.mapWithDefault(React.string("--"), phoneNumber =>
              <Link.Tel phoneNumber />
            ),
        },
        {
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => React.string(r.city->Option.getWithDefault("--")),
        },
        {
          name: "Created On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
        },
      ]
    />
  </Page>
}
