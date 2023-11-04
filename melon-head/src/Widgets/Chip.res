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

module ApplicationStatus = {
  @react.component
  let make = (~value: RemoteData.t<ApplicationData.status, 'e>) => {
    <span className={styles["appStatus"]}>
      {switch value {
      | Success(status) =>
        React.string(
          switch status {
          | Unverified => "Verification Pending"
          | Processing => "Processing"
          | Accepted => "Accepted"
          | Rejected => "Rejected"
          | Invalid => "Invalid"
          },
        )
      | Loading => React.string("..")
      | _ => React.null
      }}
    </span>
  }
}

module MemberStatus = {
  @react.component
  let make = (~value: RemoteData.t<MemberData.status, 'e>) => {
    <span className={styles["appStatus"]}>
      {switch value {
      | Success(status) =>
        React.string(
          switch status {
          | NewMember => "New Member"
          | CurrentMember => "Current Member"
          | PastMember => "Past Member"
          },
        )
      | Loading => React.string("..")
      | _ => React.null
      }}
    </span>
  }
}
