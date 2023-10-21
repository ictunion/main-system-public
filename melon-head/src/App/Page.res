open Belt

@module external styles: {..} = "./Page/styles.module.scss"

@react.component
let make = (
  ~children: React.element,
  ~requireAnyRole=[],
  ~mainResource: option<Api.webData<'a>>=?,
) => {
  open RemoteData
  open Api

  let session = React.useContext(SessionContext.context)

  <main className={styles["root"]}>
    {switch mainResource {
    | Some(Failure(ApiError({code, description, reason}))) => if code == 404 {
        <ErrorPage.NotFound />
      } else {
        <ErrorPage.Shared code title=reason description />
      }
    | _ => switch session {
      | Idle => React.null
      | Success(session) =>
        if Array.length(requireAnyRole) == 0 {
          children
        } else if Array.some(requireAnyRole, role => session->Session.hasRole(~role)) {
          children
        } else {
          <ErrorPage.Forbidden roles={requireAnyRole} />
        }
      | Loading =>
        <div className={styles["loading"]}>
          <Icons.Loading variant=Icons.Dark />
        </div>
      | Failure(err) => <ErrorPage.Unauthorized error={Api.showError(err)} />
      }
    }}
  </main>
}

module Title = {
  @react.component
  let make = (~children: React.element) => <h1 className={styles["title"]}> children </h1>
}

module BackButton = {
  @react.component
  let make = (~name: string, ~path: string) => {
    let onClick = _ => {
      RescriptReactRouter.push(path)
    }

    <a className={styles["backBtn"]} onClick> {React.string("🠔 Back to " ++ name)} </a>
  }
}
