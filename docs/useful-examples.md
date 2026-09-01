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
