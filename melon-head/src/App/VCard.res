/* vCard 3.0 rather than 4.0 -- it is the version both iOS and Android contacts
   import reliably. */
open Data
open Belt

let escape = (str: string) =>
  str
  ->Js.String2.split("\\")
  ->Js.Array2.joinWith("\\\\")
  ->Js.String2.split(",")
  ->Js.Array2.joinWith("\\,")
  ->Js.String2.split(";")
  ->Js.Array2.joinWith("\\;")
  ->Js.String2.split("\n")
  ->Js.Array2.joinWith("\\n")

let render = (
  ~firstName: option<string>,
  ~lastName: option<string>,
  ~phoneNumber: option<PhoneNumber.t>,
  ~email: option<Email.t>,
  ~note: option<string>,
) => {
  let first = firstName->Option.getWithDefault("")
  /* "ICT" goes into the surname itself, not the suffix field -- phones build
     the display name from N, and suffix rendering differs between iOS and
     Android. It keeps union contacts apart from personal ones. */
  let last = Js.String2.trim(lastName->Option.getWithDefault("") ++ " ICT")
  let fullName = Js.String2.trim(first ++ " " ++ last)

  [
    Some("BEGIN:VCARD"),
    Some("VERSION:3.0"),
    Some("N:" ++ escape(last) ++ ";" ++ escape(first) ++ ";;;"),
    Some("FN:" ++ escape(fullName)),
    phoneNumber->Option.map(p => "TEL;TYPE=CELL:" ++ escape(PhoneNumber.toString(p))),
    email->Option.map(e => "EMAIL;TYPE=INTERNET:" ++ escape(Email.toString(e))),
    note->Option.map(n => "NOTE:" ++ escape(n)),
    Some("END:VCARD"),
  ]
  ->Array.keepMap(x => x)
  ->Js.Array2.joinWith("\r\n") ++ "\r\n"
}

module DownloadButton = {
  @react.component
  let make = (~filename, ~firstName, ~lastName, ~phoneNumber, ~email, ~note) => {
    let onClick = _ =>
      Download.vcard(~filename, ~content=render(~firstName, ~lastName, ~phoneNumber, ~email, ~note))
    <Button onClick> {React.string("Download vCard")} </Button>
  }
}

module Member = {
  @react.component
  let make = (~detail: MemberData.detail) => {
    let number = Int.toString(detail.memberNumber)
    <DownloadButton
      filename={"member-" ++ number ++ ".vcf"}
      firstName=detail.firstName
      lastName=detail.lastName
      phoneNumber=detail.phoneNumber
      email=detail.email
      note=Some("Member #" ++ number)
    />
  }
}

module Application = {
  @react.component
  let make = (~detail: ApplicationData.detail) => {
    let id = detail.id->Uuid.toString->Js.String2.slice(~from=0, ~to_=8)
    <DownloadButton
      filename={"application-" ++ id ++ ".vcf"}
      firstName=detail.firstName
      lastName=detail.lastName
      phoneNumber=detail.phoneNumber
      email=detail.email
      note=None
    />
  }
}
