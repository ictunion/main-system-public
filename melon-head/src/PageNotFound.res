@module external styles: {..} = "./PageNotFound/styles.module.scss"

@react.component
let make = () =>
  <Page>
    <h1 className={styles["title"]}>
      <span className={styles["status-code"]}> {React.string("404")} </span>
      {React.string("Page Not Found!")}
    </h1>
    <p className={styles["desc"]}> {React.string("Page you're looking for does not exist.")} </p>
    <p className={styles["desc"]}>
      {React.string("Have a look into navigation if you can find what you're looking for.")}
    </p>
    <p className={styles["desc"]}>
      {React.string(
        "If you still can't find the page you need, make sure you're loged in under an account with right permissions!",
      )}
    </p>
  </Page>
