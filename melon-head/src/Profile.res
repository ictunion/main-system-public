@module external styles: {..} = "./Profile/styles.module.scss"

module ShowRole = {
  @react.component
  let make = (~role: Session.orcaRole) => {
    <span className={styles["role"]}> {React.string(Session.showOrcaRole(role))} </span>
  }
}

@react.component
let make = (
  ~closeProfile: JsxEvent.Mouse.t => unit,
  ~session: Api.webData<Session.t>,
  ~config: Config.t,
  ~keycloak: Keycloak.t,
) =>
  <div className={styles["root"]}>
    <div onClick={closeProfile} />
    <aside className={styles["sidePane"]}>
      <header className={styles["header"]}>
        <Icons.Profile variant=Icons.Light />
        <h4 className={styles["nav-title"]}> {React.string("Profile")} </h4>
        <button onClick={closeProfile} className={styles["closeBtn"]}>
          <Icons.Close />
        </button>
      </header>
      <section className={styles["profileDetail"]}>
        <h4 className={styles["profile-title"]}> {React.string("Profile details")} </h4>
        {switch session {
        | Success(ses) =>
          <table className={styles["profileTable"]}>
            <tbody>
              <tr title={ses.tokenClaims.sub->Data.Uuid.toString}>
                <td> {React.string("sub")} </td>
                <td>
                  <span className={styles["shorten"]}>
                    {React.string(ses.tokenClaims.sub->Data.Uuid.toString)}
                  </span>
                </td>
              </tr>
              <tr>
                <td> {React.string("email")} </td>
                <td>
                  <span className={styles["shorten"]}> {React.string("TBA")} </span>
                </td>
              </tr>
              <tr>
                <td> {React.string("name")} </td>
                <td>
                  <span className={styles["shorten"]}> {React.string("TBA")} </span>
                </td>
              </tr>
              <tr>
                <td> {React.string("roles")} </td>
                <td>
                  {React.array(
                    Array.map(
                      r => {<ShowRole role={r} key={Session.showOrcaRole(r)} />},
                      ses.tokenClaims.orcaRoles,
                    ),
                  )}
                </td>
              </tr>
            </tbody>
          </table>
        | Loading =>
          <div className={styles["loading"]}>
            <Icons.Loading />
          </div>
        | Idle => React.string("Application error")
        | Failure(err) => React.string(Api.showError(err))
        }}
      </section>
      <nav>
        <ul className={styles["navList"]}>
          <li key="1">
            <a href={config.profileUrl} target="_blank"> {React.string("Profile")} </a>
          </li>
          <li key="2">
            <a href={Config.keycloakAccountLink(config)} target="_blank">
              {React.string("Account Settings")}
            </a>
          </li>
          <li key="3" className={styles["navDivider"]} />
          <li key="4" className={styles["centeredLi"]}>
            <button className={styles["signOutBtn"]} onClick={_ => Keycloak.logout(keycloak)}>
              {React.string("Sign Out")}
            </button>
          </li>
        </ul>
      </nav>
    </aside>
  </div>
