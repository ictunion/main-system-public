open Vitest

describe("RemoteData", () => {
  test("map transforms only Success", t => {
    t
    ->expect(RemoteData.Success(1)->RemoteData.map(x => x + 1))
    ->Expect.toEqual(RemoteData.Success(2))
    t->expect(RemoteData.Loading->RemoteData.map(x => x + 1))->Expect.toEqual(RemoteData.Loading)
  })

  test("unwrap falls back to default", t => {
    t->expect(RemoteData.Success(2)->RemoteData.unwrap(~default=0, x => x * 10))->Expect.toBe(20)
    t->expect(RemoteData.Failure("e")->RemoteData.unwrap(~default=0, x => x * 10))->Expect.toBe(0)
  })

  test("toOption", t => {
    t->expect(RemoteData.Success(1)->RemoteData.toOption)->Expect.toEqual(Some(1))
    t->expect(RemoteData.Idle->RemoteData.toOption)->Expect.toEqual(None)
  })
})
