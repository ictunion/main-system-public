import { test, expect } from "@playwright/test";
import { fixture, ids, login, mockApi, session } from "./support";

test("admin opens a workplace from the list and edits it", async ({ page }) => {
  await login(page);
  const workplace = fixture("workplace_summary");
  const api = await mockApi(page, {
    "GET /session/current": session(["list-members", "list-workplaces", "manage-workplaces"]),
    "GET /stats/workplaces/basic": fixture("stats_workplaces"),
    "GET /workplaces/active": [workplace],
    [`GET /workplaces/${ids.workplace}`]: workplace,
    [`GET /workplaces/${ids.workplace}/representatives`]: [],
    [`PATCH /workplaces/${ids.workplace}`]: (req) => ({ ...workplace, ...req.postDataJSON() }),
  });

  await page.goto("/workplaces");
  await expect(page.getByText("acme@example.com")).toBeVisible();

  await page.locator(`a[href="/workplaces/${ids.workplace}"]`).click();
  await expect(page).toHaveURL(`/workplaces/${ids.workplace}`);
  await expect(page.getByText("Ulice 1, Praha")).toBeVisible();

  await page.getByRole("button", { name: "Edit", exact: true }).click();
  await page.getByLabel("Short Name", { exact: true }).fill("ACME CZ");
  await page.getByLabel("Company Address", { exact: true }).fill("Nová 2, Brno");
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page.getByRole("button", { name: "Save" })).toBeHidden();
  await expect(page.getByText("Nová 2, Brno")).toBeVisible();
  const patch = api.calls.find((c) => c.method === "PATCH");
  // Unchanged fields are sent as they were; Orca keeps a stored value on null.
  expect(patch?.body).toEqual({
    newsletter_id: 7,
    workplace_short_name: "ACME CZ",
    company_id: "12345678",
    company_address: "Nová 2, Brno",
  });
});
