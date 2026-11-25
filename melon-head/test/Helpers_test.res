open Vitest

@module("react-dom/server")
external renderToStaticMarkup: React.element => string = "renderToStaticMarkup"

// Visible text of an element, markup stripped.
let text = el => el->renderToStaticMarkup->String.replaceRegExp(/<[^>]*>/g, "")

describe("MemberSummaryTable.viewPaddedNumber", () => {
  test("pads to 7 digits", t => {
    t->expect(MemberSummaryTable.viewPaddedNumber(42, ())->text)->Expect.toBe("0000042")
    t->expect(MemberSummaryTable.viewPaddedNumber(1234567, ())->text)->Expect.toBe("1234567")
  })

  test("does not cut numbers longer than 7 digits", t => {
    t->expect(MemberSummaryTable.viewPaddedNumber(12345678, ())->text)->Expect.toBe("12345678")
  })

  test("marks representatives with a star", t => {
    let view = isRepresentative =>
      MemberSummaryTable.viewPaddedNumber(42, ~isRepresentative, ())->text
    t->expect(view(Some(true)))->Expect.toBe("0000042 ★")
    t->expect(view(Some(false)))->Expect.toBe("0000042")
    t->expect(view(None))->Expect.toBe("0000042")
  })
})

describe("YearMonthFilter.fromSearch", () => {
  let parse = search => YearMonthFilter.fromSearch(~search, ~currentYear=2026)

  test("reads year and month", t => {
    t->expect(parse("year=2024&month=3"))->Expect.toEqual((2024, Some(3)))
  })

  test("defaults to the current year and all months", t => {
    t->expect(parse(""))->Expect.toEqual((2026, None))
    t->expect(parse("month=all"))->Expect.toEqual((2026, None))
  })

  test("refuses years outside firstYear..currentYear", t => {
    t->expect(parse("year=2019"))->Expect.toEqual((2026, None))
    t->expect(parse("year=2027"))->Expect.toEqual((2026, None))
    t->expect(parse("year=2020"))->Expect.toEqual((2020, None))
  })

  test("refuses months outside 1..12", t => {
    t->expect(parse("month=0"))->Expect.toEqual((2026, None))
    t->expect(parse("month=13"))->Expect.toEqual((2026, None))
    t->expect(parse("month=x"))->Expect.toEqual((2026, None))
    t->expect(parse("month=12"))->Expect.toEqual((2026, Some(12)))
  })

  test("does not mix up keys that share a prefix", t => {
    t->expect(parse("yearly=2024&year=2023"))->Expect.toEqual((2023, None))
  })
})

describe("Data.Day", () => {
  test("round-trips a calendar day without a timezone shift", t => {
    t->expect(Data.Day.fromString("2026-01-31")->Data.Day.toString)->Expect.toBe("2026-01-31")
    t->expect(Data.Day.fromString("2026-01-31")->Date.getDate)->Expect.toBe(31)
  })
})

describe("OfficeData", () => {
  test("every title round-trips through its API text", t => {
    OfficeData.allTitles->Array.forEach(
      title =>
        t
        ->expect(title->OfficeData.titleToString->OfficeData.titleFromString)
        ->Expect.toEqual(Some(title)),
    )
    t->expect(OfficeData.titleFromString("Secretary"))->Expect.toEqual(None)
  })

  test("addYears rolls 29 February over to 1 March", t => {
    let d = Data.Day.fromString("2024-02-29")->OfficeData.addYears(1)
    t->expect(d->Data.Day.toString)->Expect.toBe("2025-03-01")
  })
})
