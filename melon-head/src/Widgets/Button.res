@module external styles: {..} = "./Button/styles.module.scss"

type btnType =
  | Normal
  | Cta
  | Danger

@react.component
let make = (~onClick: JsxEvent.Mouse.t => unit, ~btnType=Normal, ~disabled=false, ~children) => {
  let className =
    styles["base"] ++
    " " ++
    switch btnType {
    | Normal => styles["normal"]
    | Cta => styles["cta"]
    | Danger => styles["danger"]
    }

  <button onClick className disabled> children </button>
}

module Panel = {
  @react.component
  let make = (~children) => {
    <nav className={styles["panel"]}> children </nav>
  }
}
