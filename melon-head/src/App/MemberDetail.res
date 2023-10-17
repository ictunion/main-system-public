@react.component
let make = () => {
  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Member")} </Page.Title>
    <Page.BackButton name="member list" path="/members" />
    {React.string("This page is not implemented yet")}
  </Page>
}
