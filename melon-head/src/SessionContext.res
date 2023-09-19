let session: Api.webData<Session.t> = RemoteData.Idle
let context = React.createContext(session)

module Provider = {
  let make = React.Context.provider(context)
}
