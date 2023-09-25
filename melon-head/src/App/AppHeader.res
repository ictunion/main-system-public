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
    <button className={styles["navBtn"]} onClick={toggleNav}>
      <Icons.Hamburger isOpen={isNavOpen} />
    </button>
    <h1 className={styles["appTitle"]} onClick={openHome}>
      <img src={melonSvg} />
      {React.string("Orca / Melon Head")}
    </h1>
    <button className={styles["profileBtn"]} onClick={openProfile}>
      <Icons.Profile />
    </button>
  </header>
}
