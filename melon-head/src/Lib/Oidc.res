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

// referenence to Oidc class
type t

@send
external getUser: t => User.t = "getCurrentUser"

@send
external signinOut: t => unit = "signOut"
