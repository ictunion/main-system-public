module Encode = {
  open Json.Encode

  let strOption = str => {
    if str == "" {
      null
    } else {
      string(str)
    }
  }

  let note = (note: string) => object([("note", strOption(note))])
}
