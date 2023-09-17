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
