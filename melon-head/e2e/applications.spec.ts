import { test, expect } from "@playwright/test";
import { fixture, ids, login, mockApi, session } from "./support";

test("admin accepts a verified application", async ({ page }) => {
  await login(page);
  const detail = fixture("application_detail");
  const api = await mockApi(page, {
    "GET /session/current": session(["list-applications", "view-application", "resolve-applications"]),
    [`GET /applications/${ids.application}`]: detail,
    [`GET /applications/${ids.application}/workplace`]: fixture("application_workplace"),
    [`GET /applications/${ids.application}/files`]: [],
    [`POST /applications/${ids.application}/accept`]: { ...detail, accepted_at: "2026-11-24T10:00:00Z" },
    "GET /applications/processing": [fixture("application_processing_summary")],
    "GET /stats/applications/basic": fixture("stats_applications"),
  });

  await page.goto(`/applications/${ids.application}`);
  await page.getByRole("button", { name: "Accept", exact: true }).click();
  // The modal repeats the label; its button is the last one rendered.
  await page.getByRole("button", { name: "Accept", exact: true }).last().click();

  // Back to the list the application came from (status before accepting).
  await expect(page).toHaveURL(/\/applications#processing$/);
  const accept = api.calls.find((c) => c.method === "POST");
  expect(accept?.path).toBe(`/applications/${ids.application}/accept`);
  // No custom number entered: Orca assigns the next one.
  expect(accept?.body).toEqual({ member_number: null });
});
