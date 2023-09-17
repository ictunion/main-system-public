@module external styles: {..} = "./Icons/styles.module.scss"
@module external melonSvg: string = "/static/svg/melon.svg"
@module external profileImg: string = "/static/png/profile.png"

type variant =
  | Dark
  | Light

module Hamburger = {
  @react.component
  let make = (~isOpen: bool) => {
    let classNames = styles["hamburger"] ++ (isOpen ? " " ++ styles["hamburger-open"] : "")

    <span className={classNames}>
      <i />
    </span>
  }
}

module Profile = {
  @react.component
  let make = (~variant: variant=Dark) => {
    let cssClass = switch variant {
    | Dark => styles["profile-dark"]
    | Light => styles["profile-light"]
    }

    <span className={cssClass}>
      <img src={profileImg} alt="profile" />
    </span>
  }
}

module Close = {
  @react.component
  let make = () => {
    <span className={styles["close"]} />
  }
}

module Loading = {
  @react.component
  let make = (~variant: variant=Light) => {
    let className = switch variant {
    | Light => styles["loading-light"]
    | Dark => styles["loading-dark"]
    }
    <div className>
      <div />
      <div />
      <div />
      <div />
    </div>
  }
}
