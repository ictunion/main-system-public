@react.component
let make = (~api, ~id, ~modal) => {
  let csTemplate: string = %raw("require('../../raw/welcome_workplace_email_cs.mjml').default")
  let enTemplate: string = %raw("require('../../raw/welcome_workplace_email_en.mjml').default")

  let preferences: MemberWelcomeEmail.welcomeEmailPreferences = {
    csTemplate,
    enTemplate,
    csSubject: "Vítej v Odborové organizaci pracujících v ICT!",
    enSubject: "Welcome to the Trade union of workers in ICT!",
    variables: _ => Js.Dict.empty(),
  }

  <MemberWelcomeEmail api id modal preferences />
}
