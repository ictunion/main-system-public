@module external styles: {..} = "./Message/styles.module.scss"

module Error = {
  @react.component
  let make = (~children) => {
    <article className={styles["error"]}> children </article>
  }
}

module Info = {
  @react.component
  let make = (~children) => {
    <article className={styles["info"]}> children </article>
  }
}

module Warning = {
  @react.component
  let make = (~children) => {
    <article className={styles["warning"]}> children </article>
  }
}

module ButtonPanel = {
  @react.component
  let make = (~children) => {
    <section className={styles["buttonPanel"]}> children </section>
  }
}

module Title = {
  @react.component
  let make = (~children) => {
    <h3 className={styles["title"]}> children </h3>
  }
}
