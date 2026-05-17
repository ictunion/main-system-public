open Belt

@react.component
let make = (~api, ~id, ~modal) => {
  let csTemplate: string = %raw("require('../../raw/welcome_email_cs.mjml').default")
  let enTemplate: string = %raw("require('../../raw/welcome_email_en.mjml').default")

  let preferences: MemberWelcomeEmail.welcomeEmailPreferences = {
    csTemplate,
    enTemplate,
    csSubject: "Vítej v Odborové organizaci pracujících v ICT!",
    enSubject: "Welcome to the ICT Union!",
    variables: detail => {
      Js.Dict.fromList(list{("variable_symbol", Int.toString(detail.memberNumber))})
    },
  }

  <MemberWelcomeEmail api id modal preferences />
}
