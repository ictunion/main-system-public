@module external styles: {..} = "./ErrorPage/styles.module.scss"

open Belt

module Shared = {
  @react.component
  let make = (~code: int, ~title: string, ~description: string, ~children: React.element) =>
    <div>
      <h1 className={styles["title"]}>
        <span className={styles["statusCode"]}> {React.string(code->Int.toString)} </span>
        {React.string(title)}
      </h1>
      <p className={styles["desc"]}> {React.string(description)} </p>
      {children}
    </div>
}

module NotFound = {
  @react.component
  let make = () =>
    <Shared code=404 title="Page Not Found!" description="Page you're looking for does not exist.">
      <p className={styles["desc"]}>
        {React.string("Have a look into navigation if you can find what you're looking for.")}
      </p>
      <p className={styles["desc"]}>
        {React.string(
          "If you still can't find the page you need, make sure you're loged in under an account with right permissions!",
        )}
      </p>
    </Shared>
}

module Forbidden = {
  @react.component
  let make = (~roles: array<Session.orcaRole>) =>
    <Shared
      code=403 title="Forbidden" description="This page requires explicit role you don't have.">
      <p className={styles["desc"]}>
        {React.string("This page requires at least one of the following roles: ")}
        {roles
        ->Array.mapWithIndex((i, role) =>
          <code className={styles["code"]} key={Int.toString(i)}>
            {React.string(role->Session.showOrcaRole)}
          </code>
        )
        ->React.array}
      </p>
    </Shared>
}

module Unauthorized = {
  @react.component
  let make = (~error: string) =>
    <Shared code=401 title="Unauthorized" description="Error loading application configuration.">
      <p className={styles["desc"]}>
        {React.string("Following error occured during application initialization:")}
        <p>
          <code className={styles["code"]}> {React.string(error)} </code>
        </p>
      </p>
    </Shared>
}
