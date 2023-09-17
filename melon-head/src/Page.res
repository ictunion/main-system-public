@module external styles: {..} = "./Page/styles.module.scss"

@react.component
let make = (~children: React.element) => <main className={styles["root"]}> children </main>

module Title = {
  @react.component
  let make = (~children: React.element) => <h1 className={styles["title"]}> children </h1>
}
