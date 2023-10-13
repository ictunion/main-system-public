@module external styles: {..} = "./Button/styles.module.scss"

type variant =
  | Normal
  | Cta
  | Danger

@react.component
let make = (
  ~onClick: option<JsxEvent.Mouse.t => unit>=?,
  ~variant=Normal,
  ~disabled=false,
  ~type_="button",
  ~children,
) => {
  let className =
    styles["base"] ++
    " " ++
    switch variant {
    | Normal => styles["normal"]
    | Cta => styles["cta"]
    | Danger => styles["danger"]
    }

  <button ?onClick type_ className disabled> children </button>
}

module Panel = {
  @react.component
  let make = (~children) => {
    <nav className={styles["panel"]}> children </nav>
  }
}
