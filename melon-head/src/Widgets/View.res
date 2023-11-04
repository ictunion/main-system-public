let option = (o: option<'a>, view: 'a => React.element) => {
  switch o {
  | Some(v) => view(v)
  | None => React.string("---")
  }
}
