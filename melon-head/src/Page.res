open Belt

@module external styles: {..} = "./Page/styles.module.scss"

@react.component
let make = (~children: React.element, ~requireAnyRole=[]) => {
  let session = React.useContext(SessionContext.context)

  <main className={styles["root"]}>
    {switch session {
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
    | Failure(err) =>
      <div className={styles["error"]}>
        <h1> {React.string("Unauthorized")} </h1>
        <p> {React.string("There was an erro while loading session data")} </p>
        <pre> {React.string(Api.showError(err))} </pre>
      </div>
    }}
  </main>
}

module Title = {
  @react.component
  let make = (~children: React.element) => <h1 className={styles["title"]}> children </h1>
}
