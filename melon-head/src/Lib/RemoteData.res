type t<'a, 'b> =
  | Success('a)
  | Failure('b)
  | Idle
  | Loading

let init = _ => Idle

let fromResult = (result: result<'a, 'b>): t<'a, 'b> => {
  switch result {
  | Ok(a) => Success(a)
  | Error(b) => Failure(b)
  }
}

let setLoading = _ => Loading

let map = (t: t<'a, 'e>, f: 'a => 'b): t<'b, 'e> => {
  switch t {
  | Success(a) => Success(f(a))
  | Loading => Loading
  | Failure(e) => Failure(e)
  | Idle => Idle
  }
}

let unwrap = (t: t<'a, 'e>, ~default: 'b, f: 'a => 'b): 'b => {
  switch t {
  | Success(a) => f(a)
  | _ => default
  }
}

let toOption = (t: t<'a, 'e>): option<'a> => {
  switch t {
  | Success(a) => Some(a)
  | _ => None
  }
}
