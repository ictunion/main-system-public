/* Gates on a config-derived feature flag (e.g. Config.bankEnabled), as
   opposed to SessionContext.RequireRole/RequireBankRole which gate on the
   viewer's Keycloak roles. A feature can be off for everyone regardless of
   role, so the two kinds of gate are kept independent and are meant to be
   composed together at call sites that need both. */
@react.component
let make = (~enabled: bool, ~children: React.element) => enabled ? children : React.null
