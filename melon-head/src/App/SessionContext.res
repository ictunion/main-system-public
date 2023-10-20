let session: Api.webData<Session.t> = RemoteData.Idle
let context = React.createContext(session)

module Provider = {
  let make = React.Context.provider(context)
}

module RequireRole = {
  open Belt

  @react.component
  let make = (~children: React.element, ~anyOf=[]) => {
    let session = React.useContext(context)

    {
      switch session {
      | Idle => React.null
      | Success(session) =>
        if Array.length(anyOf) == 0 {
          children
        } else if Array.some(anyOf, role => session->Session.hasRole(~role)) {
          children
        } else {
          React.null
        }
      | Loading => React.null
      | Failure(_) => React.null
      }
    }
  }
}
