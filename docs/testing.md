# Orca tests

Endpoint tests send real requests through the Axum router. Keycloak and the background
queue are replaced by test fakes. Tests that need a database, or read `gray-whale/migrations` or `melon-head/test/fixtures` (absent from the Nix build source), are marked `#[ignore]`.

| File (`orca/src/api/`) | What it checks |
|---|---|
| `test_support.rs` | Shared harness: `TestApp`, fake tokens, test databases, data helpers |
| `auth_tests.rs` | Every route: no token → 401, wrong role → 403, right role passes |
| `scope_tests.rs` | Workplace reps see and resolve only their own workplace |
| `mutation_tests.rs` | Every POST/PUT/PATCH/DELETE succeeds and returns what it changed |
| `contract_tests.rs` | Melon Head's fixtures match what Orca sends: each is read into its response struct and serialized back unchanged |
| `edge_tests.rs` | Conflicts (409), missing records (404), invalid input (400/422), unknown route/method (404/405), unreadable bodies (400/413/415), Keycloak not configured (404) |

## Running locally

```bash
make up    # Postgres + Keycloak
make test-all                    # Orca + Melon Head unit and Playwright tests, from the repo root
# or by hand:
cd orca
cargo test                       # tests that need no database
DATABASE_URL=postgres://postgres:superuser@localhost:5432/ictunion SQLX_OFFLINE=true \
  cargo test -- --include-ignored  # all tests
```

- `DATABASE_URL` must be a superuser. The tests migrate a template database once per run,
  then give each test its own copy (`orca_test_*`).
- The app inside the tests logs in as `orca`, as in production, so a missing `GRANT` fails.
- Each test drops its database when it ends, even if it fails. Only `orca_test_template`
  stays; the next run recreates it. Don't run two test runs against one server at once.
- `SQLX_OFFLINE=true` makes the `query!` macros use the `.sqlx` cache instead of the database.

## CI (`.github/workflows/orca.yaml`)

- `nix-build` runs `nix flake check`: build, clippy, fmt, and the tests that need no database.
- `db-tests` starts a `postgres:15.2-alpine` container and runs `cargo test -- --include-ignored`.

# Melon Head tests

Vitest tests written in ReScript (`rescript-vitest`), in `melon-head/test/`. They need no
backend, Keycloak or `config.json`.

| File (`melon-head/test/`) | What it checks |
|---|---|
| `fixtures/*.json` | Response bodies as Orca (and the bank API) send them. Orca's `contract_tests.rs` fails when an Orca fixture no longer matches its struct; the `payment_*` ones (bank API) are not checked |
| `Fixture.res`, `Render.res` | Shared helpers: load and edit fixtures, server-render with a given session |
| `Decoders_test.res` | Every decoder in `src/Data/` reads its fixture and refuses a missing or mistyped field |
| `Helpers_test.res` | `viewPaddedNumber`, `YearMonthFilter`, `Data.Day`, office titles |
| `Gates_test.res` | `RequireRole`, `RequireBankRole`, `RequireFeature`, `Page` roles, admin vs rep on `WorkplaceDetail` |

## Running locally

```bash
cd melon-head
npm test            # rescript build && vitest run
npx vitest          # watch mode; run `npm run watch` alongside to recompile .res files
```

Vitest runs the compiled `test/*_test.bs.js`, so a `.res` change needs a ReScript build first.

## Playwright flows

Browser tests in `melon-head/e2e/`. They start the dev server themselves (`rescript build`
and `webpack-dev-server` on :1234, or reuse one already running) and need `config.json`. No Orca
or Keycloak: `support.ts` seeds a fake `oidc-client-ts` user in `localStorage` and stubs
every `/api/*` request (Orca and the bank API) with the fixtures above. Roles come from the stubbed
`/session/current`, so a test picks its roles with `session([...])`.

| File (`melon-head/e2e/`) | Flow |
|---|---|
| `members.spec.ts` | Member list to member detail; adding a note sends a `PATCH …/note`; the Payments tab shows bank API payments |
| `applications.spec.ts` | Admin accepts a verified application |
| `workplaces.spec.ts` | Workplace list to detail; editing sends the changed fields in a `PATCH` |
| `missing-dues.spec.ts` | Missing dues lists a member from the bank API; adding and deleting a dues note |
| `workplace-rep.spec.ts` | A rep sees only their workplace and is refused on `/members` |

```bash
cd melon-head
nix develop ..#melon-head         # NixOS: Chromium from nixpkgs (playwright-driver.browsers)
npx playwright install chromium   # other systems, once
npm run e2e                       # npx playwright test --ui for the interactive runner
```

An unstubbed request gets a 404 in Orca's error shape.

## CI (`.github/workflows/melon-head.yaml`)

- `nix-build` builds the bundle and checks formatting.
- `test` runs `npm ci` and `npm test` on Node 24.
- `e2e` runs `npm ci` and `npm run e2e` inside `nix develop .#melon-head` (Chromium from nixpkgs,
  as locally), after copying `config.example.json`. Traces are uploaded when it fails.
