@module external styles: {..} = "./Link/styles.module.scss"

module Email = {
  @react.component
  let make = (~email: Data.Email.t) => {
    let emailString = Data.Email.toString(email)
    <a className={styles["email"]} href={"mailto:" ++ emailString}> {React.string(emailString)} </a>
  }
}

module Tel = {
  @react.component
  let make = (~phoneNumber: Data.PhoneNumber.t) => {
    open Js

    let phoneString = Data.PhoneNumber.toString(phoneNumber)

    // Most phone numbers should have this format
    // so we can as well format it in more readable way.
    // Otherwise we just use the string directly
    let formatted = if phoneString->String.length == 13 {
      phoneString->String.slice(~from=0, ~to_=4) ++
      " " ++
      phoneString->String.slice(~from=4, ~to_=7) ++
      " " ++
      phoneString->String.slice(~from=7, ~to_=10) ++
      " " ++
      phoneString->String.slice(~from=10, ~to_=13)
    } else {
      phoneString
    }

    <a className={styles["tel"]} href={"tel:" ++ phoneString}> {React.string(formatted)} </a>
  }
}

module Uuid = {
  @react.component
  let make = (~uuid: Data.Uuid.t, ~toPath: string => string) => {
    let shorten = uuid->Data.Uuid.toString->Js.String.substrAtMost(~from=0, ~length=7) ++ "..."
    let openPath = (_: JsxEvent.Mouse.t) => {
      RescriptReactRouter.push(toPath(uuid->Data.Uuid.toString))
    }
    <a className={styles["uuid"]} onClick={openPath}>
      {React.string(shorten)}
      <span className={styles["uuidFull"]}> {React.string(Data.Uuid.toString(uuid))} </span>
    </a>
  }
}
