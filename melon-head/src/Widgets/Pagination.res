@module external styles: {..} = "./Pagination/styles.module.scss"

/* Client side pagination over data which is already loaded. Pages are 0-based. */

let pageCount = (~total: int, ~pageSize: int): int => (total + pageSize - 1) / pageSize

/* Keeps `page` in range when the data shrinks (e.g. after a refetch) */
let clamp = (page: int, ~pageCount: int): int => Js.Math.max_int(0, Js.Math.min_int(page, pageCount - 1))

let slice = (items: array<'a>, ~page: int, ~pageSize: int): array<'a> =>
  items->Belt.Array.slice(~offset=page * pageSize, ~len=pageSize)

/* Renders nothing when everything fits on one page */
@react.component
let make = (~page: int, ~pageCount: int, ~onChange: int => unit) => {
  if pageCount <= 1 {
    React.null
  } else {
    <nav className={styles["pagination"]}>
      <Button disabled={page <= 0} onClick={_ => onChange(page - 1)}>
        {React.string("‹ Previous")}
      </Button>
      <span className={styles["status"]}>
        {React.string(
          "Page " ++ Belt.Int.toString(page + 1) ++ " of " ++ Belt.Int.toString(pageCount),
        )}
      </span>
      <Button disabled={page >= pageCount - 1} onClick={_ => onChange(page + 1)}>
        {React.string("Next ›")}
      </Button>
    </nav>
  }
}
