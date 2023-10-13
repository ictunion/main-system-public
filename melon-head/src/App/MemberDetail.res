@react.component
let make = () => {
  let backToMembers = _ => {
    RescriptReactRouter.push("/members")
  }

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Member")} </Page.Title>
    <a onClick=backToMembers> {React.string("🠔 Back to member list")} </a>
    {React.string("This page is not implemented yet")}
  </Page>
}
