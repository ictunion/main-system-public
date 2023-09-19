@module external styles: {..} = "./Chip/styles.module.scss"

open Belt

module Count = {
  @react.component
  let make = (~value: RemoteData.t<int, 'e>) => {
    <span className={styles["count"]}>
      {switch value {
      | Success(count) => React.string(Int.toString(count))
      | Loading => React.string("..")
      | _ => React.null
      }}
    </span>
  }
}
