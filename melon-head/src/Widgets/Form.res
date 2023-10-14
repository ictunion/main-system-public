@module external styles: {..} = "./Form/styles.module.scss"

@send external focus: Dom.element => unit = "focus"

open Belt

/* Bind to react-datepicker */
%%raw("import 'react-datepicker/dist/react-datepicker.css'")
module DatePicker = {
  @react.component @module("react-datepicker")
  external make: (
    ~selected: option<Js.Date.t>,
    ~onChange: Js.Date.t => unit,
    ~dateFormat: string,
    ~placeholderText: string,
    ~dropdownMode: string,
    ~peekNextMonth: option<unit>=?,
    ~showMonthDropdown: option<unit>=?,
    ~showYearDropdown: option<unit>=?,
    ~isClearable: option<unit>=?,
  ) => React.element = "default"
}

@react.component
let make = (~onSubmit: JsxEvent.Form.t => unit, ~children): React.element => {
  let doSubmit = event => {
    ReactEvent.Form.preventDefault(event)
    onSubmit(event)
  }
  <form onSubmit=doSubmit className={styles["form"]}> children </form>
}

module Label = {
  @react.component
  let make = (~children): React.element => <label className={styles["label"]}> children </label>
}

module HorizontalLabel = {
  @react.component
  let make = (~children): React.element =>
    <label className={styles["horizontalLabel"]}> children </label>
}

module Field = {
  @react.component
  let make = (~children): React.element => <div className={styles["field"]}> children </div>
}

let viewOption = (o: option<'a>, view: 'a => React.element) => {
  switch o {
  | Some(v) => view(v)
  | None => React.null
  }
}

module Checkbox = {
  @react.component
  let make = (~checked, ~onChange): React.element => <input type_="checkbox" checked onChange />
}

module TextField = {
  @react.component
  let make = (
    ~label: string,
    ~placeholder: string,
    ~value,
    ~onInput: string => unit,
  ): React.element => {
    let setValue = event => {
      let newVal = ReactEvent.Form.currentTarget(event)["value"]
      onInput(newVal)
    }

    <Field>
      <Label>
        {React.string(label)}
        <input placeholder value onInput=setValue />
      </Label>
    </Field>
  }
}

module MemberNumber = {
  let inputEmpyErr = Some("Member number can't be empty.")
  let validate = (number: option<string>): option<string> => {
    switch number {
    // Automatically assigning member number
    | None => None
    // Explicitely specifying member number
    | Some(num) =>
      switch Int.fromString(num) {
      | None =>
        if num == "" {
          inputEmpyErr
        } else {
          Some("Expects number, got `" ++ num ++ "`.")
        }
      | Some(i) =>
        if i <= 0 {
          Some("Member number must be positive number (greater than `0`).")
        } else {
          None
        }
      }
    }
  }
  @react.component
  let make = (~number: option<string>, ~setMemberNumber: option<string> => unit): React.element => {
    let hasCustomNumber = Option.isSome(number)

    let toggleHasCustomNumber = _ => {
      switch number {
      | Some(_) => setMemberNumber(None)
      | None => setMemberNumber(Some(""))
      }
    }

    // Validations
    let (validationErr, setValidationErr) = React.useState(() => inputEmpyErr)

    // Handle new custom number input values
    let onInput = (event: JsxEvent.Form.t) => {
      let newVal = ReactEvent.Form.currentTarget(event)["value"]
      setMemberNumber(Some(newVal))

      setValidationErr(_ => validate(newVal))
    }

    // DOM node of text input
    let inputEl = React.useRef(Js.Nullable.null)

    // Focus input
    React.useEffect1(() => {
      if hasCustomNumber {
        inputEl.current->Js.Nullable.toOption->Belt.Option.forEach(input => input->focus)
      }
      None
    }, [number])

    <div className={styles["memberNumber"]}>
      <Field>
        <HorizontalLabel>
          <Checkbox checked={!hasCustomNumber} onChange=toggleHasCustomNumber />
          <strong> {React.string("Automatically assign member number")} </strong>
        </HorizontalLabel>
      </Field>
      {switch number {
      | Some(memberNumber) =>
        <div className={styles["customMemberNumber"]}>
          <Field>
            <Label>
              {React.string("Member Number:")}
              <br />
              <input
                ref={ReactDOM.Ref.domRef(inputEl)} value=memberNumber onInput placeholder="42"
              />
            </Label>
            {validationErr->viewOption(str => {
              <Message.Error> {React.string(str)} </Message.Error>
            })}
          </Field>
        </div>
      | None =>
        <Message.Info>
          {React.string(
            "New member will be automatically assigned a number that is one greater (+1) than the ",
          )}
          <strong> {React.string("highest existing member number")} </strong>
          {React.string(".")}
        </Message.Info>
      }}
    </div>
  }
}

module LanguageSelect = {
  type language = {
    code: string,
    name: string,
    flag: string,
  }

  let languages = [
    {
      code: "",
      name: "[unknown]",
      flag: "🏴‍☠️",
    },
    {
      code: "cs",
      name: "Czech",
      flag: "🇨🇿",
    },
    {
      code: "en",
      name: "English",
      flag: "🇬🇧",
    },
  ]

  let viewOption = l => {
    <option key=l.code value=l.code> {React.string(l.flag ++ " " ++ l.name)} </option>
  }

  // This is not very type-safe but hope is that eventually we revamp how we deal with these
  @react.component
  let make = (~value: string="cs", ~onChange) => {
    let selectLanguage = (event: JsxEvent.Form.t) => {
      let newVal = ReactEvent.Form.currentTarget(event)["value"]
      onChange(newVal)
    }

    <select value onChange={selectLanguage}>
      {React.array(Array.map(languages, viewOption))}
    </select>
  }
}

module Date = {
  @react.component
  let make = (~value: option<Js.Date.t>, ~onChange, ~label: string) => {
    <Field>
      <Label>
        {React.string(label)}
        // wrap in div so vendor css and js are not confused
        <div>
          <DatePicker
            selected=value
            onChange
            placeholderText="1988/01/31"
            dateFormat="yyyy/MM/dd"
            dropdownMode="select"
            peekNextMonth=Some()
            showMonthDropdown=Some()
            showYearDropdown=Some()
            isClearable=Some()
          />
        </div>
      </Label>
    </Field>
  }
}
