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
    note: Some(""),
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

let newNoteModal = (~api, ~modal, ~refreshMembers, uuid, ~isApplication, initialNote): Modal.modalContent => {
  title: "Update note",
  content: <NewNote api modal refreshMembers uuid isApplication initialNote />,
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

let urlToTab = (url: RescriptReactRouter.url): option<MemberData.status> => {
  open MemberData

  switch url.hash {
  | "all" => None
  | "current" => Some(CurrentMember)
  | "past" => Some(PastMember)
  | _ => Some(NewMember)
  }
}

let tabToUrl = (tab: option<MemberData.status>): string => {
  open MemberData

  switch tab {
  | None => "/members#all"
  | Some(NewMember) => "/members#new"
  | Some(CurrentMember) => "/members#current"
  | Some(PastMember) => "/members#past"
  }
}

let columns: array<DataTable.column<MemberData.summary>> = [
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
    view: r => r.leftAt->View.option(a => React.string(Js.Date.toLocaleDateString(a))),
  },
  {
    name: "First Name",
    minMax: ("150px", "2fr"),
    view: r => r.firstName->View.option(React.string),
  },
  {
    name: "Last Name",
    minMax: ("150px", "2fr"),
    view: r => r.lastName->View.option(React.string),
  },
  {
    name: "Email",
    minMax: ("250px", "2fr"),
    view: r => r.email->View.option(email => <Link.Email email />),
  },
  {
    name: "Phone",
    minMax: ("220px", "2fr"),
    view: r => r.phoneNumber->View.option(phoneNumber => <Link.Tel phoneNumber />),
  },
  {
    name: "Last Company",
    minMax: ("220px", "2fr"),
    view: r => r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
  },
  {
    name: "City",
    minMax: ("250px", "1fr"),
    view: r => r.city->View.option(React.string),
  },
  {
    name: "Created On",
    minMax: ("150px", "1fr"),
    view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
  },
]

// We could use Functor module to generate these but would probably make it harder for people to understand what is going on

module All = {
  @react.component
  let make = (~api, ~modal) => {
    let (members, _, refreshMembers) =
      api->Hook.getData(~path="/members", ~decoder=Json.Decode.array(MemberData.Decode.summary))

    let openNewMembersTab = _ => {
      RescriptReactRouter.push(Some(MemberData.NewMember)->tabToUrl)
    }

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshMembers, uuid, ~isApplication=false, note))

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
          view: r => r.leftAt->View.option(a => React.string(Js.Date.toLocaleDateString(a))),
        },
        {
          name: "First Name",
          minMax: ("150px", "2fr"),
          view: r => r.firstName->View.option(React.string),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => r.lastName->View.option(React.string),
        },
        {
          name: "Note",
          minMax: ("250px", "10fr"),
          view: r =>
            <a onClick={_ => openNewNoteModal(r.id, r.note)}>
              {
                let note = if Option.getWithDefault(r.note, "Add note") == "" {
                  "Add note"
                } else {
                  Option.getWithDefault(r.note, "Add note")
                }

                React.string(note)
              }
            </a>,
        },
        {
          name: "Last Company",
          minMax: ("220px", "2fr"),
          view: r =>
            r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
        },
        {
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Created On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
        },
      ]>
      <p> {React.string("Currently there are no member.")} </p>
      <p>
        <small>
          {React.string(
            "To create a member you'll need confirm (and generate account) for some of ",
          )}
          <a onClick=openNewMembersTab> {React.string("new members")} </a>
          {React.string(".")}
        </small>
      </p>
    </DataTable>
  }
}

module New = {
  @react.component
  let make = (~api, ~modal) => {
    let (members, _, refreshMembers) =
      api->Hook.getData(~path="/members/new", ~decoder=Json.Decode.array(MemberData.Decode.summary))

    let openNewMemberModal = _ =>
      Modal.Interface.openModal(modal, newMemberModal(~api, ~modal, ~refreshMembers))

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshMembers, uuid, ~isApplication=false, note))

    <div className={styles["membersTab"]}>
      <Button.Panel>
        <Button onClick=openNewMemberModal> {React.string("Add New Member")} </Button>
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
            view: r => r.leftAt->View.option(a => React.string(Js.Date.toLocaleDateString(a))),
          },
          {
            name: "First Name",
            minMax: ("150px", "2fr"),
            view: r => r.firstName->View.option(React.string),
          },
          {
            name: "Last Name",
            minMax: ("150px", "2fr"),
            view: r => r.lastName->View.option(React.string),
          },
          {
            name: "Note",
            minMax: ("250px", "10fr"),
            view: r =>
              <a onClick={_ => openNewNoteModal(r.id, r.note)}>
                {
                  let note = if Option.getWithDefault(r.note, "Add note") == "" {
                    "Add note"
                  } else {
                    Option.getWithDefault(r.note, "Add note")
                  }

                  React.string(note)
                }
              </a>,
          },
          {
            name: "Last Company",
            minMax: ("220px", "2fr"),
            view: r =>
              r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
          },
          {
            name: "City",
            minMax: ("250px", "1fr"),
            view: r => r.city->View.option(React.string),
          },
          {
            name: "Created On",
            minMax: ("150px", "1fr"),
            view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
          },
        ]>
        <p> {React.string("There are members who need to be onboarded.")} </p>
        <p>
          <small>
            {React.string("Maybe you want to ")}
            <a onClick=openNewMemberModal> {React.string("create a new one")} </a>
            {React.string("?")}
          </small>
        </p>
      </DataTable>
    </div>
  }
}

module Current = {
  @react.component
  let make = (~api, ~modal) => {
    let (members, _, refreshMembers) =
      api->Hook.getData(
        ~path="/members/current",
        ~decoder=Json.Decode.array(MemberData.Decode.summary),
      )

    let openNewMembersTab = _ => {
      RescriptReactRouter.push(Some(MemberData.NewMember)->tabToUrl)
    }

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshMembers, uuid, ~isApplication=false, note))

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
          name: "First Name",
          minMax: ("150px", "2fr"),
          view: r => r.firstName->View.option(React.string),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => r.lastName->View.option(React.string),
        },
        {
          name: "Note",
          minMax: ("250px", "10fr"),
          view: r =>
            <a onClick={_ => openNewNoteModal(r.id, r.note)}>
              {
                let note = if Option.getWithDefault(r.note, "Add note") == "" {
                  "Add note"
                } else {
                  Option.getWithDefault(r.note, "Add note")
                }

                React.string(note)
              }
            </a>,
        },
        {
          name: "Last Company",
          minMax: ("220px", "2fr"),
          view: r =>
            r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
        },
        {
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Created On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
        },
      ]>
      <p> {React.string("There are no members yet.")} </p>
      <p>
        <small>
          {React.string(
            "To create a member you'll need confirm (and generate account) for some of ",
          )}
          <a onClick=openNewMembersTab> {React.string("new members")} </a>
          {React.string(".")}
        </small>
      </p>
    </DataTable>
  }
}

module Past = {
  @react.component
  let make = (~api, ~modal) => {
    let (members, _, refreshMembers) =
      api->Hook.getData(
        ~path="/members/past",
        ~decoder=Json.Decode.array(MemberData.Decode.summary),
      )

    let openNewNoteModal = (uuid, note) =>
      Modal.Interface.openModal(modal, newNoteModal(~api, ~modal, ~refreshMembers, uuid, ~isApplication=false, note))

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
          view: r => r.leftAt->View.option(a => React.string(Js.Date.toLocaleDateString(a))),
        },
        {
          name: "First Name",
          minMax: ("150px", "2fr"),
          view: r => r.firstName->View.option(React.string),
        },
        {
          name: "Last Name",
          minMax: ("150px", "2fr"),
          view: r => r.lastName->View.option(React.string),
        },
        {
          name: "Note",
          minMax: ("250px", "10fr"),
          view: r =>
            <a onClick={_ => openNewNoteModal(r.id, r.note)}>
              {
                let note = if Option.getWithDefault(r.note, "Add note") == "" {
                  "Add note"
                } else {
                  Option.getWithDefault(r.note, "Add note")
                }

                React.string(note)
              }
            </a>,
        },
        {
          name: "Last Company",
          minMax: ("220px", "2fr"),
          view: r =>
            r.companyNames->Array.get(0)->Option.flatMap(a => a)->View.option(React.string),
        },
        {
          name: "City",
          minMax: ("250px", "1fr"),
          view: r => r.city->View.option(React.string),
        },
        {
          name: "Created On",
          minMax: ("150px", "1fr"),
          view: r => React.string(r.createdAt->Js.Date.toLocaleDateString),
        },
      ]>
      <p> {React.string("There are no ex-members.")} </p>
    </DataTable>
  }
}

@react.component
let make = (~api: Api.t, ~modal: Modal.Interface.t) => {
  let (activeTab, setActiveTab) = React.useState(_ =>
    RescriptReactRouter.dangerouslyGetInitialUrl()->urlToTab
  )

  let _ = RescriptReactRouter.watchUrl(url => {
    setActiveTab(_ => urlToTab(url))
  })

  let tabHandlers = (
    activeTab,
    f => {
      let newTab = f(activeTab)
      RescriptReactRouter.push(tabToUrl(newTab))
    },
  )

  let (basicStats, _, _) =
    api->Hook.getData(~path="/stats/members/basic", ~decoder=StatsData.Members.Decode.basic)

  <Page requireAnyRole=[ListMembers]>
    <Page.Title> {React.string("Members")} </Page.Title>
    <Tabbed.Tabs>
      <Tabbed.Tab value=Some(MemberData.NewMember) handlers=tabHandlers>
        <span> {React.string("New")} </span>
        <Chip.Count value={basicStats->RemoteData.map(r => r.new)} />
      </Tabbed.Tab>
      <Tabbed.Tab
        value=Some(MemberData.CurrentMember) handlers=tabHandlers color=Some("var(--color6)")>
        <span> {React.string("Current")} </span>
        <Chip.Count value={basicStats->RemoteData.map(r => r.current)} />
      </Tabbed.Tab>
      <Tabbed.Tab
        value=Some(MemberData.PastMember) handlers=tabHandlers color=Some("var(--color7)")>
        <span> {React.string("Past")} </span>
        <Chip.Count value={basicStats->RemoteData.map(r => r.past)} />
      </Tabbed.Tab>
      <Tabbed.TabSpacer />
      <Tabbed.Tab value=None handlers=tabHandlers color=Some("var(--color1)")>
        <span> {React.string("All")} </span>
        <Chip.Count value={basicStats->RemoteData.map(StatsData.Members.all)} />
      </Tabbed.Tab>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Some(MemberData.NewMember) handlers={tabHandlers}>
      <New api modal />
    </Tabbed.Content>
    <Tabbed.Content tab=Some(MemberData.CurrentMember) handlers={tabHandlers}>
      <Current api modal />
    </Tabbed.Content>
    <Tabbed.Content tab=Some(MemberData.PastMember) handlers={tabHandlers}>
      <Past api modal />
    </Tabbed.Content>
    <Tabbed.Content tab=None handlers={tabHandlers}>
      <All api modal />
    </Tabbed.Content>
  </Page>
}
