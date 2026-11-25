// Server-renders an element (no effects run, so nothing is fetched) and returns its markup.
@module("react-dom/server") external markup: React.element => string = "renderToStaticMarkup"

// Visible text, markup stripped.
let text = el => el->markup->String.replaceRegExp(/<[^>]*>/g, "")

let session = (~orca=[], ~bank=[]): Session.t => {
  tokenClaims: {
    sub: Data.Uuid.unsafeFromString("00000000-0000-4000-8000-000000000002"),
    email: Data.Email.unsafeFromString("user@example.com"),
    name: Some("User"),
    orcaRoles: orca,
    bankRoles: bank,
  },
  memberId: None,
}

let withSession = (session: Api.webData<Session.t>, children) =>
  <SessionContext.Provider value=session> children </SessionContext.Provider>
