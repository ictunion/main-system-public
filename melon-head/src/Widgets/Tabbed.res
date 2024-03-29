@module external styles: {..} = "./Tabbed/styles.module.scss"

let make = default => {
  React.useState(_ => default)
}

module Tabs = {
  @react.component
  let make = (~children) => {
    <div className={styles["scrollContainer"]}>
      <nav className={styles["nav"]}> children </nav>
    </div>
  }
}

module Tab = {
  @react.component
  let make = (~children, ~value, ~handlers, ~color=None) => {
    let (active, setActive) = handlers
    let activate = _ => setActive(_ => value)

    let cssClass = if active == value {
      styles["activeTab"]
    } else {
      styles["tab"]
    }

    let style = switch color {
    | None => ReactDOM.Style.make()
    | Some(activeColor) =>
      ReactDOM.Style.make()->ReactDOM.Style.unsafeAddProp("--active-color", activeColor)
    }

    <button onClick={activate} className={cssClass} style> children </button>
  }
}

module TabSpacer = {
  @react.component
  let make = () => {
    <div className={styles["tabSpacer"]} />
  }
}

module Content = {
  @react.component
  let make = (~children=React.null, ~tab, ~handlers) => {
    if fst(handlers) == tab {
      <div> children </div>
    } else {
      React.null
    }
  }
}
