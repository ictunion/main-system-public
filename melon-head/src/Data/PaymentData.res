type coveredMonth = {
  year: int,
  month: int,
}

type transaction = {
  processedTransactionId: int,
  transactionDate: Js.Date.t,
  amount: string,
  currency: string,
  coveredMonths: array<coveredMonth>,
}

/* A member with unpaid dues, as returned by the bank API's
   /payments/{year}/missing and /payments/{year}/{month}/missing endpoints.
   totalMissedMonths counts the member's unpaid months across their whole
   liability window (not just the queried period); rows arrive sorted by it,
   descending. */
type missingPaymentMember = {
  memberNumber: int,
  totalMissedMonths: int,
  hasEverPaid: bool,
}

/* A transaction an admin left a comment on, as returned by the bank API's
   /payments/{year}/commented and /payments/{year}/{month}/commented
   endpoints -- same shape as `transaction` but tied to a member and carrying
   the comment instead of the covered months. */
type commentedTransaction = {
  memberNumber: int,
  processedTransactionId: int,
  transactionDate: Js.Date.t,
  amount: string,
  currency: string,
  adminComment: string,
}

module Decode = {
  open Json.Decode

  let coveredMonth = object(field => {
    year: field.required(. "year", int),
    month: field.required(. "month", int),
  })

  let transaction = object(field => {
    processedTransactionId: field.required(. "processed_transaction_id", int),
    transactionDate: field.required(. "transaction_date", date),
    amount: field.required(. "amount", string),
    currency: field.required(. "currency", string),
    coveredMonths: field.required(. "covered_months", array(coveredMonth)),
  })

  let history = array(transaction)

  let missingPaymentMember = object(field => {
    memberNumber: field.required(. "member_number", int),
    totalMissedMonths: field.required(. "total_missed_months", int),
    hasEverPaid: field.required(. "has_ever_paid", bool),
  })

  let missing = array(missingPaymentMember)

  let commentedTransaction = object(field => {
    memberNumber: field.required(. "member_number", int),
    processedTransactionId: field.required(. "processed_transaction_id", int),
    transactionDate: field.required(. "transaction_date", date),
    amount: field.required(. "amount", string),
    currency: field.required(. "currency", string),
    adminComment: field.required(. "admin_comment", string),
  })

  let commented = array(commentedTransaction)
}
