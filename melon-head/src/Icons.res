@module external styles: {..} = "./Icons/styles.module.scss"
@module external melonSvg: string = "/static/svg/melon.svg"
@module external profileImg: string = "/static/png/profile.png"

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
  type variant =
    | Dark
    | Light

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
  let make = () => {
    <div className={styles["loading"]}>
      <div />
      <div />
      <div />
      <div />
    </div>
  }
}
