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
