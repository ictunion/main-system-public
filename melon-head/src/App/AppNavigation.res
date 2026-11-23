@module external styles: {..} = "./AppNavigation/styles.module.scss"

module NavSeprator = {
  @react.component
  let make = () => <li className={styles["separator"]} />
}

// Renders its separator before the link only when the link itself is visible.
module NavItem = {
  @react.component
  let make = (
    ~path: string,
    ~text: string,
    ~session: Session.t,
    ~requiredRole=None,
    ~requiredBankRole=None,
    ~withSeparator=true,
  ) => {
    let openRoute = (_: JsxEvent.Mouse.t) => {
      RescriptReactRouter.push(path)
    }

    let hasRequiredRole = switch requiredRole {
    | None => true
    | Some(role) => Session.hasRole(session, ~role)
    }

    let hasRequiredBankRole = switch requiredBankRole {
    | None => true
    | Some(role) => Session.hasBankRole(session, ~role)
    }

    let accessible = hasRequiredRole && hasRequiredBankRole

    let isOpen = switch RescriptReactRouter.useUrl().path {
    | list{} => path == "/"
    | list{route} => "/" ++ route == path
    | list{route, _} => "/" ++ route == path
    | _ => false
    }

    let className = styles["navItem"] ++ (isOpen ? " " ++ styles["navItemOpen"] : "")

    if accessible {
      <>
        {withSeparator ? <NavSeprator /> : React.null}
        <li className onClick={openRoute}>
          <a> {React.string(text)} </a>
        </li>
      </>
    } else {
      React.null
    }
  }
}

@react.component
let make = (~isOpen: bool, ~session: Api.webData<Session.t>) => {
  if isOpen {
    <nav className={styles["root"]}>
      {switch session {
      | Success(session) =>
        <ul className={styles["navList"]}>
          <NavItem key="1" path="/" text="Dashboard" session withSeparator=false />
          <NavItem
            key="2"
            path="/applications"
            text="Applications"
            session
            requiredRole=Some(Session.ListApplications)
          />
          <NavItem
            key="3" path="/members" text="Members" session requiredRole=Some(Session.ListMembers)
          />
          <NavItem
            key="4"
            path="/workplaces"
            text="Workplaces"
            session
            requiredRole=Some(Session.ManageWorkplaces)
          />
          <NavItem
            key="5"
            path="/missing-dues"
            text="Missing Dues"
            session
            requiredBankRole=Some(Session.PaymentHistory)
          />
          <NavItem
            key="6"
            path="/my-workplace-applications"
            text="Workplace Applications"
            session
            requiredRole=Some(Session.ListOwnWorkplaceApplications)
          />
          <NavItem
            key="7"
            path="/my-workplace"
            text="Workplace Members"
            session
            requiredRole=Some(Session.ListOwnWorkplaceMembers)
          />
          <NavItem
            key="8"
            path="/my-workplace-missing-dues"
            text="Workplace Missing Dues"
            session
            requiredRole=Some(Session.ListOwnWorkplaceMembers)
            requiredBankRole=Some(Session.WorkplacePaymentHistory)
          />
          <NavItem
            key="9"
            path="/my-workplace-settings"
            text="Workplace Info"
            session
            requiredRole=Some(Session.ListOwnWorkplace)
          />
          <NavItem
            key="10"
            path="/elected-offices"
            text="Elected Offices"
            session
            requiredRole=Some(Session.ListMembers)
          />
          <NavItem key="11" path="/settings" text="Settings" session />
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
