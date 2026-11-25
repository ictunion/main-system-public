import { Page, Request } from "@playwright/test";
import { readFileSync } from "node:fs";
import config from "../config.json";

export const fixture = (name: string): any =>
  JSON.parse(readFileSync(`${__dirname}/../test/fixtures/${name}.json`, "utf8"));

export const ids = {
  member: "00000000-0000-4000-8000-000000000001",
  application: "00000000-0000-4000-8000-000000000010",
  workplace: "00000000-0000-4000-8000-000000000020",
};

// Orca's /session/current with the given roles; roles drive every gate in the UI.
export const session = (orca: string[], bank: string[] = []) => {
  const s = fixture("session");
  s.token_claims.resource_access = {
    orca: { roles: orca },
    "bank-system": { roles: bank },
  };
  return s;
};

/* Skip the Keycloak redirect: oidc-client-ts reads the user from localStorage
   (index.ts sets WebStorageStateStore), so seeding it is enough. The token is
   never checked because every API call is stubbed. */
export const login = async (page: Page) => {
  const key = `oidc.user:${config.keycloak_url}/realms/${config.keycloak_realm}:${config.keycloak_client_id}`;
  const user = {
    access_token: "test-token",
    token_type: "Bearer",
    scope: "openid profile email",
    profile: { sub: "00000000-0000-4000-8000-000000000002" },
    expires_at: Math.floor(Date.now() / 1000) + 24 * 3600,
  };
  await page.addInitScript(
    ({ k, v }) => window.localStorage.setItem(k, v),
    { k: key, v: JSON.stringify(user) },
  );
  // Anything reaching Keycloak means the bypass broke; fail loudly.
  await page.route(`${config.keycloak_url}/**`, (route) => route.abort());
};

export type Call = { method: string; path: string; body: unknown };

/* Stub Orca and the bank API. `routes` maps "METHOD /path" (path without the
   /api prefix) to a response body, or to a function of the request. Anything
   else gets a 404 in Orca's error shape. */
export const mockApi = async (
  page: Page,
  routes: Record<string, unknown | ((req: Request) => unknown)>,
) => {
  const calls: Call[] = [];
  await page.route("**/api/**", async (route) => {
    const req = route.request();
    const path = new URL(req.url()).pathname.replace(/^\/api/, "");
    const key = `${req.method()} ${path}`;
    calls.push({ method: req.method(), path, body: req.postDataJSON() });
    if (key in routes) {
      const r = routes[key];
      const body = typeof r === "function" ? r(req) : r;
      return route.fulfill({ json: body });
    }
    return route.fulfill({
      status: 404,
      json: { error: { code: 404, description: "Not Found", reason: `no stub for ${key}` } },
    });
  });
  return { calls };
};
