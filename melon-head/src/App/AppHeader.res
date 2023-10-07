@module external styles: {..} = "./AppHeader/styles.module.scss"
@module external melonSvg: string = "/static/svg/melon-head.svg"

@react.component
let make = (
  ~toggleNav: JsxEvent.Mouse.t => unit,
  ~openProfile: JsxEvent.Mouse.t => unit,
  ~isNavOpen: bool,
) => {
  let openHome = _ => RescriptReactRouter.push("/")
  let rootClass = styles["root"] ++ (isNavOpen ? " " ++ styles["withNav"] : "")
  let navBtnClass = styles["navBtn"] ++ (isNavOpen ? " " ++ styles["navBtnClose"] : "")
  <header className=rootClass>
    <button className=navBtnClass onClick=toggleNav>
      <div className={styles["navBtnInner"]}>
        <Icons.Hamburger isOpen=isNavOpen />
      </div>
    </button>
    <h1 className={styles["appTitle"]} onClick=openHome>
      <img src=melonSvg className={styles["logo"]} />
      {React.string("Orca / Melon Head")}
    </h1>
    <button className={styles["profileBtn"]} onClick=openProfile>
      <Icons.Profile />
    </button>
  </header>
}
