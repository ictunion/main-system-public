@module external styles: {..} = "./Icons/styles.module.scss"
@module external melonSvg: string = "/static/svg/melon.svg"
@module external profileImg: string = "/static/svg/profile.svg"

type variant =
  | Dark
  | Light

module Hamburger = {
  @react.component
  let make = (~isOpen: bool) => {
    let classNames = styles["hamburger"] ++ (isOpen ? " " ++ styles["hamburgerOpen"] : "")

    <span className={classNames}>
      <i />
    </span>
  }
}

module Profile = {
  @react.component
  let make = (~variant: variant=Dark) => {
    let cssClass = switch variant {
    | Dark => styles["profileDark"]
    | Light => styles["profileLight"]
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
    | Light => styles["loadingLight"]
    | Dark => styles["loadingDark"]
    }
    <div className>
      <div />
      <div />
      <div />
      <div />
    </div>
  }
}
