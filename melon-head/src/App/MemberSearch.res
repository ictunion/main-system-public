open Data

@send external focus: Dom.element => unit = "focus"

// `#123 Jane Doe`, as the browser lists each member under the typed text.
let label = (m: MemberData.summary): string => {
  let name = [m.firstName, m.lastName]->Array.filterMap(x => x)->Array.joinUnsafe(" ")
  "#" ++ Int.toString(m.memberNumber) ++ " " ++ name
}

// The member whose ID is `value`, the way a pick from the list fills it in.
let findById = (members: array<MemberData.summary>, value: string) => {
  let id = value->String.trim->String.toLowerCase
  members->Array.find(m => Uuid.toString(m.id) == id)
}

/* Text input that finds a member by name, member number or ID. The browser
   matches what's typed against each member's label in a datalist; picking a
   member fills in their ID, which callers resolve with `findById`. A pasted
   ID resolves the same way. `listId` must be unique on the page. */
@react.component
let make = (
  ~listId: string,
  ~members: array<MemberData.summary>,
  ~value: string,
  ~onChange: string => unit,
  ~placeholder="Jane Doe",
) => {
  /* After a pick from the list, the browser can keep drawing the placeholder
     over whatever is typed next if the caller clears the input. A fresh input
     element (new `key`) after each pick avoids that; focus is put back so
     typing carries on. */
  let (inputKey, setInputKey) = React.useState(() => 0)
  let inputRef = React.useRef(Nullable.null)
  let refocus = React.useRef(false)

  React.useEffect1(() => {
    if refocus.current {
      refocus.current = false
      inputRef.current->Nullable.toOption->Option.forEach(focus)
    }
    None
  }, [inputKey])

  let onChange = event => {
    let value = ReactEvent.Form.currentTarget(event)["value"]
    onChange(value)
    if findById(members, value)->Option.isSome {
      refocus.current = true
      setInputKey(k => k + 1)
    }
  }

  <>
    <input
      key={Int.toString(inputKey)}
      ref={ReactDOM.Ref.domRef(inputRef)}
      type_="text"
      list=listId
      placeholder
      value
      onChange
    />
    <datalist id=listId>
      {members
      ->Array.map(m =>
        <option key={Uuid.toString(m.id)} value={Uuid.toString(m.id)}>
          {React.string(label(m))}
        </option>
      )
      ->React.array}
    </datalist>
  </>
}
