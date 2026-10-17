# Useful Examples

Ad-hoc commands and snippets that come up repeatedly during development. Not a
full API reference — just the things worth not re-deriving every time.

## Submit a registration request (new application)

`POST /registration/join` on Orca (`:8000`). Public endpoint, no auth. Creates a
registration request and queues a confirmation email to the background processor.

```bash
curl -i -X POST http://localhost:8000/registration/join \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "jakub@example.com",
    "first_name": "Jakub",
    "last_name": "Test",
    "date_of_birth": "1990-01-01",
    "address": "Testovaci 1",
    "city": "Praha",
    "postal_code": "11000",
    "phone_number": "+420123456789",
    "company_name": "ICT Union Test s.r.o.",
    "occupation": "Software Engineer",
    "local": "cz",
    "signature": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
  }'
```

### Field notes

Defined in [`orca/src/api/registration/mod.rs`](../orca/src/api/registration/mod.rs) (`RegistrationRequest`).

| Field | Required | Notes |
|-------|----------|-------|
| `email` | yes | Must be a valid email address. |
| `first_name`, `last_name` | yes | Non-empty. |
| `date_of_birth` | yes | `YYYY-MM-DD`. Sent as JSON string; `null` is rejected by the validator. |
| `city` | yes | Non-empty. |
| `phone_number` | yes | Non-empty. |
| `company_name` | yes | Non-empty. |
| `occupation` | yes | Non-empty. |
| `local` | yes | Exactly 2 characters (regional local code, e.g. `cz`). |
| `signature` | yes | `data:image/<type>;base64,<data>` string. Must decode to a real image — it is loaded by the `image` crate and resized to 492x192. The value above is a valid 1x1 PNG. |
| `address` | no | — |
| `postal_code` | no | — |

Success returns `200` with a confirmation payload. The confirmation token is
generated server-side.

## Generate a workplace document (PDF)

`POST /workplaces/<id>/files` on Orca (`:8000`). Requires `manage-workplaces`.
Answers `202` and generates the PDF in the background processor; it then shows
up in `GET /workplaces/<id>/files`. `document_type` selects the document, the
other fields are its inputs.

```bash
TOKEN=$(./scripts/get-token.sh <client> <user> <pass>)
curl -i -X POST http://localhost:8000/workplaces/<workplace-uuid>/files \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "document_type": "entrustment",
    "workplace_short_name": "ACME CZ",
    "meeting_date": "2026-10-03",
    "company_address": "Vinohradská 1234/56, 120 00 Praha 2",
    "company_id": "12345678"
  }'
```

`entrustment` also accepts optional `"removed_member_ids": ["<member-uuid>", ...]`;
when not empty, the PDF gets an extra section listing members who are no longer
on the committee as of the meeting date (unknown ids are a `409`).
`member_confirmation` takes only the short name (used in the file name):
`{"document_type": "member_confirmation", "workplace_short_name": "ACME CZ"}`.
`workplace_announcement` takes the same fields as `entrustment` except `meeting_date` and `removed_member_ids`.

Every document needs `union_chairperson_member_id` in `Rocket.toml` and at
least one representative on the workplace (otherwise `409`). The workplace
announcement also needs `union_chairperson_email`.

## Refresh the sqlx offline query cache

Run from `orca/` (inside `nix develop .#orca`, for the `cargo sqlx` subcommand)
after adding or changing a `query!`/`query_as!` macro call. Needed because the
repo checks in a `.sqlx` directory so CI can compile without a live database;
skipping this after a query change means CI builds against stale metadata.
This also recompiles the crate against the live DB, so it doubles as a normal
rebuild.

```bash
DATABASE_URL="postgres://orca@localhost/ictunion" cargo sqlx prepare
```

`orca` is a real role in the dev database with no password, since the
`postgres` container runs with `POSTGRES_HOST_AUTH_METHOD=trust` -- same
connection string as the `postgres` field in `Rocket.toml`.
