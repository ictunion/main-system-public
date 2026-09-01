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
          <NavSeprator key="4" />
          <NavItem
            key="5" path="/members" text="Members" session requiredRole=Some(Session.ListMembers)
          />
          <NavSeprator key="6" />
          <NavItem
            key="7"
            path="/workplaces"
            text="Workplaces"
            session
            requiredRole=Some(Session.ManageWorkplaces)
          />
          <NavSeprator key="8" />
          <NavItem
            key="9"
            path="/my-workplace"
            text="My Workplace"
            session
            requiredRole=Some(Session.ListOwnWorkplaceMembers)
          />
          <NavSeprator key="10" />
          <NavItem
            key="11"
            path="/my-workplace-settings"
            text="My Workplace Settings"
            session
            requiredRole=Some(Session.ListOwnWorkplace)
          />
          <NavSeprator key="12" />
          <NavItem key="13" path="/settings" text="Settings" session />
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
