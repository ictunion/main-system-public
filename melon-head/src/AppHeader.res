@module external styles: {..} = "./AppHeader/styles.module.scss"
@module external melonSvg: string = "/static/svg/melon.svg"

@react.component
let make = (
  ~toggleNav: JsxEvent.Mouse.t => unit,
  ~openProfile: JsxEvent.Mouse.t => unit,
  ~isNavOpen: bool,
) => {
  let openHome = _ => RescriptReactRouter.push("/")
  <header className={styles["root"]}>
    <button className={styles["nav-btn"]} onClick={toggleNav}>
      <Icons.Hamburger isOpen={isNavOpen} />
    </button>
    <h1 className={styles["app-title"]} onClick={openHome}>
      <img src={melonSvg} />
      {React.string("Orca's Melon Head Manager")}
    </h1>
    <button className={styles["profile-btn"]} onClick={openProfile}>
      <Icons.Profile />
    </button>
  </header>
}
