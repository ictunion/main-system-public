@module external styles: {..} = "./AppNavigation/styles.module.scss"

module NavItem = {
  @react.component
  let make = (~path: string, ~text: string, ~session: Session.t, ~requiredRole=None) => {
    let openRoute = (_: JsxEvent.Mouse.t) => {
      RescriptReactRouter.push(path)
    }

    let accessible = switch requiredRole {
    | None => true
    | Some(role) => Session.hasRole(session, ~role)
    }

    let isOpen = switch RescriptReactRouter.useUrl().path {
    | list{} => path == "/"
    | list{route} => "/" ++ route == path
    | list{route, _} => "/" ++ route == path
    | _ => false
    }

    let className = styles["navItem"] ++ (isOpen ? " " ++ styles["navItemOpen"] : "")

    if accessible {
      <li className onClick={openRoute}>
        <a> {React.string(text)} </a>
      </li>
    } else {
      React.null
    }
  }
}

module NavSeprator = {
  @react.component
  let make = () => <li className={styles["separator"]} />
}

@react.component
let make = (~isOpen: bool, ~session: Api.webData<Session.t>) => {
  if isOpen {
    <nav className={styles["root"]}>
      {switch session {
      | Success(session) =>
        <ul className={styles["navList"]}>
          <NavItem key="1" path="/" text="Dashboard" session />
          <NavSeprator key="2" />
          <NavItem
            key="3"
            path="/applications"
            text="Applications"
            session
            requiredRole=Some(Session.ListApplications)
          />
          /* <NavItem */
          /* key="4" */
          /* path="/checklist" */
          /* text="Checklist" */
          /* session */
          /* requiredRole=Some(Session.ListApplications) */
          /* /> */
          /* <NavSeprator key="5" /> */
          /* <NavItem */
          /* key="6" */
          /* path="/members" */
          /* text="Members" */
          /* session */
          /* requiredRole=Some(Session.ListApplications) */
          /* /> */
          /* <NavItem */
          /* key="7" */
          /* path="/workplaces" */
          /* text="Workplaces" */
          /* session */
          /* requiredRole=Some(Session.ListApplications) */
          /* /> */
          /* <NavItem */
          /* key="8" */
          /* path="/keycloak-mappings" */
          /* text="Login Mapping" */
          /* session */
          /* requiredRole=Some(Session.ListApplications) */
          /* /> */
          /* <NavSeprator key="9" /> */
          /* <NavItem */
          /* key="10" */
          /* path="/settings" */
          /* text="Settings" */
          /* session */
          /* requiredRole=Some(Session.ListApplications) */
          /* /> */
        </ul>
      | Loading =>
        <div className={styles["loading"]}>
          <Icons.Loading />
        </div>
      | _ => React.null
      }}
    </nav>
  } else {
    React.null
  }
}
