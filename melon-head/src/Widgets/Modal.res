@module external styles: {..} = "./Modal/styles.module.scss"

type modalContent = {
  title: string,
  content: React.element,
}

// This almost looks like OOP but
// but hiding react effects into opaque types is not fun
module Interface = {
  type t = {
    closeModal: unit => unit,
    openModal: modalContent => unit,
    state: option<modalContent>,
  }

  let state = t => t.state
  let openModal = (t, modalContent) => t.openModal(modalContent)
  let closeModal = t => t.closeModal()
}

let use = (): Interface.t => {
  let (state: option<modalContent>, setModal) = React.useState(_ => None)

  let openModal = content => {
    setModal(_ => Some(content))
  }

  let closeModal = _ => {
    setModal(_ => None)
  }

  {closeModal, openModal, state}
}

// TODO: Maybe this should take an interface?
@react.component
let make = (~title: string, ~children: React.element, ~close: JsxEvent.Mouse.t => unit) => {
  <div className={styles["container"]}>
    <div className={styles["overlay"]} onClick=close />
    <div className={styles["box"]}>
      <header className={styles["header"]}>
        {React.string(title)}
        <button className={styles["closeBtn"]} onClick=close>
          <Icons.Close variant=Icons.Light />
        </button>
      </header>
      <main className={styles["body"]}> {children} </main>
    </div>
  </div>
}

module Content = {
  @react.component
  let make = (~children: React.element) => {
    <main className={styles["content"]}> children </main>
  }
}
