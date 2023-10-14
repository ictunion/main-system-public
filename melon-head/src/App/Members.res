@module external styles: {..} = "./Members/styles.module.scss"

open Belt

module NewMember = {
  open MemberData

  let emptyMember = {
    memberNumber: Some(""),
    firstName: "",
    lastName: "",
    dateOfBirth: None,
    email: "",
    phoneNumber: "",
    address: "",
    city: "",
    postalCode: "",
    language: "",
  }

  @react.component
  let make = (~api: Api.t, ~modal: Modal.Interface.t, ~refreshMembers) => {
    let (newMember, setNewMember) = React.useState(_ => emptyMember)
    let disabled = Form.MemberNumber.validate(newMember.memberNumber)->Option.isSome
    let (createMore, setCreateMore) = React.useState(_ => false)
    let (error, setError) = React.useState(() => None)

    let onSubmit = _ => {
      let body: Js.Json.t = MemberData.Encode.newMember(newMember)
      let req = api->Api.postJson(~path="/members", ~decoder=MemberData.Decode.summary, ~body)

      req->Future.get(res => {
        switch res {
        | Ok(_data) => {
            let _ = refreshMembers()
            if createMore {
              setNewMember(_ => {
                ...emptyMember,
                memberNumber: newMember.memberNumber
                ->Option.flatMap(Int.fromString)
                ->Option.map(n => Int.toString(n + 1)),
              })
            } else {
              Modal.Interface.closeModal(modal)
            }
          }
        | Error(e) => setError(_ => Some(e))
        }
      })
    }

    <Form onSubmit>
      <Form.MemberNumber
        number=newMember.memberNumber
        setMemberNumber={memberNumber => setNewMember(member => {...member, memberNumber})}
      />
      <Form.TextField
        label="Email"
        placeholder="member@union.org"
        value=newMember.email
        onInput={email => setNewMember(m => {...m, email})}
      />
      <Form.Field>
        <Form.Label>
          {React.string("Language")}
          <Form.LanguageSelect
            value=newMember.language onChange={language => setNewMember(m => {...m, language})}
          />
        </Form.Label>
      </Form.Field>
      <Form.TextField
        label="First Name"
        placeholder="Jane"
        value=newMember.firstName
        onInput={firstName => setNewMember(m => {...m, firstName})}
      />
      <Form.TextField
        label="Last Name"
        placeholder="Doe"
        value=newMember.lastName
        onInput={lastName => setNewMember(m => {...m, lastName})}
      />
      <Form.Date
        label="Date of Birth"
        value=newMember.dateOfBirth
        onChange={date => setNewMember(m => {...m, dateOfBirth: Some(date)})}
      />
      <Form.TextField
        label="Phone Number"
        placeholder="+420 777 666 555"
        value=newMember.phoneNumber
        onInput={phoneNumber => setNewMember(m => {...m, phoneNumber})}
      />
      <Form.TextField
        label="Address"
        placeholder="Elm Street 3"
        value=newMember.address
        onInput={address => setNewMember(m => {...m, address})}
      />
      <Form.TextField
        label="City"
        placeholder="London City"
        value=newMember.city
        onInput={city => setNewMember(m => {...m, city})}
      />
      <Form.TextField
        label="Postal Code"
        placeholder="E1 0AA"
        value=newMember.postalCode
        onInput={postalCode => setNewMember(m => {...m, postalCode})}
      />
      <Button.Panel>
        <Button
          type_="button" variant=Button.Danger onClick={_ => modal->Modal.Interface.closeModal}>
          {React.string("Cancel")}
        </Button>
        <Button type_="submit" disabled variant=Button.Cta>
          {React.string("Create New User")}
        </Button>
        <Form.HorizontalLabel>
          <Form.Checkbox checked=createMore onChange={_ => setCreateMore(v => !v)} />
          {React.string("Create another member")}
        </Form.HorizontalLabel>
      </Button.Panel>
      {switch error {
      | None => React.null
      | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
      }}
    </Form>
  }
}

let newMemberModal = (~api, ~modal, ~refreshMembers): Modal.modalContent => {
  title: "Add New Member",
  content: <NewMember api modal refreshMembers />,
}

let viewPaddedNumber = (n: int): React.element => {
  let stringified = Int.toString(n)
  let prefix = "0000000"
  let shortenedPrefix = prefix->Js.String.slice(~from=0, ~to_=7 - String.length(stringified))

  <span>
    <span className={styles["numberPrefix"]}> {React.string(shortenedPrefix)} </span>
    {React.string(stringified)}
  </span>
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  let (members, _, refreshMembers) =
    api->Hook.getData(~path="/members", ~decoder=Json.Decode.array(MemberData.Decode.summary))

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Members")} </Page.Title>
    <Button.Panel>
      <Button
        onClick={_ =>
          Modal.Interface.openModal(modal, newMemberModal(~api, ~modal, ~refreshMembers))}>
        {React.string("Add New Member")}
      </Button>
    </Button.Panel>
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
          view: r => viewPaddedNumber(r.memberNumber),
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
