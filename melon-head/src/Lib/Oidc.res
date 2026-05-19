module User = {
  type profile = {
    sub: string,
    name: option<string>,
    email: option<string>,
  }

  type t = {
    id_token: option<string>,
    access_token: string,
    refresh_token: option<string>,
    token_type: string,
    scope: string,
    profile: profile,
    expires_at: int,
    expired: bool,
  }

  let isExpired = (user: t) => user.expired
  let getToken = (user: t) => user.access_token
}

// We construct this from the type script initializer so there is no need
// for binding constructor in rescript
type userManager

@send
external getUser: userManager => promise<Js.nullable<User.t>> = "getUser"

@send
external signinOutRedirect: userManager => unit = "sigignoutRedirect"
