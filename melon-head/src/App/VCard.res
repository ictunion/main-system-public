/* vCard 3.0 rather than 4.0 -- it is the version both iOS and Android contacts
 import reliably. */
open Data

let escape = (str: string) =>
  str
  ->String.split("\\")
  ->Array.joinUnsafe("\\\\")
  ->String.split(",")
  ->Array.joinUnsafe("\\,")
  ->String.split(";")
  ->Array.joinUnsafe("\\;")
  ->String.split("\n")
  ->Array.joinUnsafe("\\n")

let render = (
  ~firstName: option<string>,
  ~lastName: option<string>,
  ~phoneNumber: option<PhoneNumber.t>,
  ~email: option<Email.t>,
  ~note: option<string>,
) => {
  let first = firstName->Option.getOr("")
  /* "ICT" goes into the surname itself, not the suffix field -- phones build
     the display name from N, and suffix rendering differs between iOS and
     Android. It keeps union contacts apart from personal ones. */
  let last = String.trim(lastName->Option.getOr("") ++ " ICT")
  let fullName = String.trim(first ++ " " ++ last)

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
  ->Array.filterMap(x => x)
  ->Array.joinUnsafe("\r\n") ++ "\r\n"
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
    let id = detail.id->Uuid.toString->String.slice(~start=0, ~end=8)
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
