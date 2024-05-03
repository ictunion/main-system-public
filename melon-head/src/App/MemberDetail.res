@module external styles: {..} = "./MemberDetail/styles.module.scss"

open Data
open Belt

// todo: we should make Note editable also in application detail
// https://github.com/ictunion/main-system/issues/164
let layout: DataGrid.t<MemberData.detail> = [
  {
    label: "Membership",
    cells: [
      {
        label: "Member Number",
        view: d => Members.viewPaddedNumber(d.memberNumber),
        minmax: ("250px", "690px"),
      },
      {
        label: "Language",
        view: d => View.option(d.language, React.string),
        minmax: ("200px", "200px"),
      },
      {
        label: "Application",
        view: d =>
          View.option(d.applicationId, uuid =>
            <a onClick={_ => RescriptReactRouter.push("/applications/" ++ Uuid.toString(uuid))}>
              {React.string(uuid->Uuid.toString)}
            </a>
          ),
        minmax: ("250px", "655px"),
      },
    ],
  },
  {
    label: "Personal Information",
    cells: [
      {
        label: "First Name",
        view: d => View.option(d.firstName, React.string),
        minmax: ("300px", "900px"),
      },
      {
        label: "Last Name",
        view: d => View.option(d.lastName, React.string),
        minmax: ("225px", "665px"),
      },
      {
        label: "Date of Birth",
        view: d => View.option(d.dateOfBirth, a => a->Js.Date.toLocaleDateString->React.string),
        minmax: ("250px", "250px"),
      },
    ],
  },
  {
    label: "Contacts",
    cells: [
      {
        label: "Email",
        view: d => View.option(d.email, email => <Link.Email email />),
        minmax: ("300px", "900px"),
      },
      {
        label: "Phone Number",
        view: d => View.option(d.phoneNumber, phoneNumber => <Link.Tel phoneNumber />),
        minmax: ("150px", "500px"),
      },
    ],
  },
  {
    label: "Address",
    cells: [
      {
        label: "Address",
        view: d => View.option(d.address, React.string),
        minmax: ("450px", "900px"),
      },
      {
        label: "City",
        view: d => View.option(d.city, React.string),
        minmax: ("150px", "500px"),
      },
      {
        label: "Postal Code",
        view: d => View.option(d.postalCode, React.string),
        minmax: ("150px", "150px"),
      },
    ],
  },
  {
    label: "Notes",
    cells: [
      {
        label: "Note",
        view: d => View.option(d.note, React.string),
        minmax: ("150px", "1500px"),
      },
    ],
  },
]

let timeRows: array<RowBasedTable.row<MemberData.detail>> = [
  ("Created", d => d.createdAt->Js.Date.toLocaleString->React.string),
  (
    "Onboarded at",
    d => View.option(d.onboardingFinishAt, a => a->Js.Date.toLocaleString->React.string),
  ),
  ("Left at", d => View.option(d.leftAt, a => a->Js.Date.toLocaleString->React.string)),
]

module Loading = {
  @react.component
  let make = () => {
    React.string("loading...")
  }
}

module Actions = {
  open MemberData

  module Accept = {
    type acceptTabs =
      | Create
      | Pair

    @react.component
    let make = (~modal, ~api, ~id, ~setDetail) => {
      let (error, setError) = React.useState(() => None)
      let tabHandlers = Tabbed.make(Create)
      let doAccept = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.patchJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/accept",
            ~decoder=MemberData.Decode.detail,
            ~body=Js.Json.null,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      let (selectedId, setId) = React.useState(_ => None)

      let selectId = (event: JsxEvent.Form.t) => {
        let newVal = ReactEvent.Form.currentTarget(event)["value"]
        setId(_ => Some(newVal))
      }

      let doPair = (_: JsxEvent.Mouse.t) => {
        let uuid = switch selectedId {
        | Some(uuid) => Json.Encode.string(uuid)
        | None => Json.Encode.null
        }
        let req =
          api->Api.patchJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/pair_oid",
            ~decoder=MemberData.Decode.detail,
            ~body=Json.Encode.object([("sub", uuid)]),
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      let (candidates: Api.webData<array<Session.user>>, _, _) =
        api->Hook.getData(
          ~path="/members/" ++ Uuid.toString(id) ++ "/list_candidate_users",
          ~decoder=Json.Decode.array(Session.Decode.user),
        )

      <Modal.Content>
        <Tabbed.Tabs>
          <Tabbed.Tab value=Create handlers=tabHandlers> {React.string("Create")} </Tabbed.Tab>
          <Tabbed.Tab value=Pair handlers=tabHandlers> {React.string("Pair Existing")} </Tabbed.Tab>
        </Tabbed.Tabs>
        <Tabbed.Content tab=Create handlers=tabHandlers>
          <div className={styles["modalBody"]}>
            <p> {React.string("Accept member and allow them to access internal resources.")} </p>
            {switch error {
            | None => React.null
            | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
            }}
          </div>
          <Button.Panel>
            <Button onClick={_ => modal->Modal.Interface.closeModal}>
              {React.string("Cancel")}
            </Button>
            <Button variant=Button.Cta onClick=doAccept> {React.string("Accept member")} </Button>
          </Button.Panel>
        </Tabbed.Content>
        <Tabbed.Content tab=Pair handlers=tabHandlers>
          <div className={styles["modalBody"]}>
            <p> {React.string("Pair existing OID account with member")} </p>
            {switch candidates {
            | Idle => <Loading />
            | Loading => <Loading />
            | Failure(err) => React.string(Api.showError(err))
            | Success(candidates) =>
              if candidates == [] {
                React.string("No candidates found")
              } else {
                <div className={styles["radioList"]}>
                  {candidates
                  ->Array.map(user => {
                    <label className={styles["radio"]}>
                      <input value={user.id->Uuid.toString} type_="radio" onInput=selectId />
                      {React.string(user.email->Data.Email.toString)}
                    </label>
                  })
                  ->React.array}
                </div>
              }
            }}
            {switch error {
            | None => React.null
            | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
            }}
          </div>
          <Button.Panel>
            <Button onClick={_ => modal->Modal.Interface.closeModal}>
              {React.string("Cancel")}
            </Button>
            <Button variant=Button.Cta onClick=doPair disabled={selectedId == None}>
              {React.string("Pair Selected")}
            </Button>
          </Button.Panel>
        </Tabbed.Content>
      </Modal.Content>
    }
  }

  module Remove = {
    @react.component
    let make = (~modal, ~api, ~id, ~setDetail) => {
      let (error, setError) = React.useState(() => None)

      let doRemove = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.deleteJson(
            ~path="/members/" ++ Uuid.toString(id),
            ~decoder=MemberData.Decode.detail,
          )

        req->Future.get(res => {
          switch res {
          | Ok(data) => {
              setDetail(_ => RemoteData.Success(data))
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })
      }

      <Modal.Content>
        <p> {React.string("Remove member and reject their access to organization resources.")} </p>
        {switch error {
        | None => React.null
        | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
        }}
        <Button.Panel>
          <Button onClick={_ => modal->Modal.Interface.closeModal}>
            {React.string("Cancel")}
          </Button>
          <Button variant=Button.Danger onClick=doRemove> {React.string("Remove Member")} </Button>
        </Button.Panel>
      </Modal.Content>
    }
  }

  let acceptModal = (~modal, ~api, ~id, ~setDetail): Modal.modalContent => {
    title: "Accept Member",
    content: <Accept modal api id setDetail />,
  }

  let removeModal = (~modal, ~api, ~id, ~setDetail): Modal.modalContent => {
    title: "Remove Member",
    content: <Remove modal api id setDetail />,
  }

  @react.component
  let make = (~status, ~modal, ~api, ~id, ~setDetail) => {
    switch status {
    | NewMember =>
      <Button.Panel>
        <Button
          variant=Button.Cta
          onClick={_ =>
            modal->Modal.Interface.openModal(acceptModal(~modal, ~api, ~id, ~setDetail))}>
          {React.string("Accept member")}
        </Button>
        <Button
          variant=Button.Danger
          onClick={_ =>
            modal->Modal.Interface.openModal(removeModal(~modal, ~api, ~id, ~setDetail))}>
          {React.string("Remove member")}
        </Button>
      </Button.Panel>
    | CurrentMember =>
      <Button.Panel>
        <Button
          variant=Button.Danger
          onClick={_ =>
            modal->Modal.Interface.openModal(removeModal(~modal, ~api, ~id, ~setDetail))}>
          {React.string("Remove member")}
        </Button>
      </Button.Panel>
    | PastMember => React.null
    }
  }
}

type tabs =
  | Metadata
  | Files
  | Occupations

let viewOccupation = (occupation: MemberData.occupation) => {
  <tr key={occupation.id->Uuid.toString}>
    <td> {occupation.companyName->View.option(React.string)} </td>
    <td> {occupation.position->View.option(React.string)} </td>
    <td> {occupation.createdAt->Js.Date.toLocaleDateString->React.string} </td>
  </tr>
}

@react.component
let make = (~api, ~id, ~modal) => {
  let (detail: Api.webData<MemberData.detail>, setDetail, _) =
    api->Hook.getData(~path="/members/" ++ Uuid.toString(id), ~decoder=MemberData.Decode.detail)

  let status = RemoteData.map(detail, MemberData.getStatus)

  let tabHandlers = Tabbed.make(Occupations)

  let (filesData, _, _) =
    api->Hook.getData(
      ~path="/members/" ++ Uuid.toString(id) ++ "/files",
      ~decoder=Json.Decode.array(Data.Decode.file),
    )

  let (occupationsData, _, _) =
    api->Hook.getData(
      ~path="/members/" ++ Uuid.toString(id) ++ "/occupations",
      ~decoder=Json.Decode.array(MemberData.Decode.occupation),
    )

  let mainOccupation = occupationsData->RemoteData.map(xs => xs->Array.get(0))

  <Page requireAnyRole=[ListMembers, ViewMember] mainResource=detail>
    <header className={styles["header"]}>
      <h1 className={styles["title"]}>
        {React.string("Member ")}
        <span className={styles["titleId"]}>
          {switch detail {
          | Success(d) => d.id->Uuid.toString->React.string
          | _ => React.string("...")
          }}
        </span>
      </h1>
      <Page.BackButton name="applications" path={status->RemoteData.toOption->Members.tabToUrl} />
      <h2 className={styles["status"]}>
        {React.string("Status:")}
        <Chip.MemberStatus value=status />
      </h2>
    </header>
    <DataGrid layout data=detail />
    <DataGrid
      data=mainOccupation
      layout={[
        {
          label: "Last Occupation",
          cells: [
            {
              label: "Company",
              view: d =>
                d
                ->Option.flatMap((a: MemberData.occupation) => a.companyName)
                ->View.option(React.string),
              minmax: ("150px", "900px"),
            },
            {
              label: "Position",
              view: d => d->Option.flatMap(a => a.position)->View.option(React.string),
              minmax: ("150px", "665px"),
            },
          ],
        },
      ]}
    />
    <Tabbed.Tabs>
      <Tabbed.Tab value=Occupations handlers=tabHandlers>
        {React.string("Occupations")}
      </Tabbed.Tab>
      <Tabbed.Tab value=Metadata handlers=tabHandlers> {React.string("Metadata")} </Tabbed.Tab>
      <Tabbed.Tab value=Files handlers=tabHandlers> {React.string("Files")} </Tabbed.Tab>
    </Tabbed.Tabs>
    <Tabbed.Content tab=Occupations handlers=tabHandlers>
      <div className={styles["occupations"]}>
        <table>
          <thead>
            <tr>
              <th> {React.string("Company")} </th>
              <th> {React.string("Position")} </th>
              <th> {React.string("Created at")} </th>
            </tr>
          </thead>
          <tbody>
            {switch occupationsData {
            | Success(occupations) => occupations->Array.map(viewOccupation)->React.array
            | _ => React.null
            }}
          </tbody>
        </table>
      </div>
    </Tabbed.Content>
    <Tabbed.Content tab=Metadata handlers=tabHandlers>
      <div className={styles["metadata"]}>
        <RowBasedTable rows=timeRows data=detail title=Some("Updates") />
      </div>
    </Tabbed.Content>
    <Tabbed.Content tab=Files handlers=tabHandlers>
      <DataGrid
        data=filesData
        layout={[
          {
            label: "",
            cells: [
              {
                label: "Files",
                minmax: ("150px,", "600px"),
                view: files => View.filesTable(~api, ~files),
              },
            ],
          },
        ]}
      />
    </Tabbed.Content>
    {switch status {
    | Success(s) => <Actions status=s modal api id setDetail />
    | _ => React.null
    }}
  </Page>
}
