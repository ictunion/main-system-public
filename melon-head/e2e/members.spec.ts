import { test, expect } from "@playwright/test";
import { fixture, ids, login, mockApi, session } from "./support";

const admin = session(["list-members", "view-member", "manage-members"]);

test.beforeEach(async ({ page }) => {
  await login(page);
});

test("member list opens the member detail", async ({ page }) => {
  await mockApi(page, {
    "GET /session/current": admin,
    "GET /stats/members/basic": fixture("stats_members"),
    "GET /members/new": [fixture("member_summary")],
    [`GET /members/${ids.member}`]: fixture("member_detail"),
  });

  await page.goto("/members");
  await expect(page.getByText("Nováková")).toBeVisible();

  await page.locator(`a[href="/members/${ids.member}"]`).click();
  await expect(page).toHaveURL(`/members/${ids.member}`);
  await expect(page.getByText("jana@example.com").first()).toBeVisible();
  await expect(page.getByText("Ulice 1")).toBeVisible();
});

test("adding a note to a member sends it to Orca", async ({ page }) => {
  const api = await mockApi(page, {
    "GET /session/current": admin,
    "GET /stats/members/basic": fixture("stats_members"),
    "GET /members/new": [fixture("member_summary")],
    [`PATCH /members/${ids.member}/note`]: { ...fixture("member_detail"), note: "Prefers email" },
  });

  await page.goto("/members");
  await page.getByText("Add note").click();
  await page.getByLabel("Note").fill("Prefers email");
  await page.getByRole("button", { name: "Update note" }).click();

  await expect(page.getByRole("button", { name: "Update note" })).toBeHidden();
  const patch = api.calls.find((c) => c.method === "PATCH");
  expect(patch).toEqual({
    method: "PATCH",
    path: `/members/${ids.member}/note`,
    body: { note: "Prefers email" },
  });
});

test("payments tab shows the member's payments from the bank API", async ({ page }) => {
  await mockApi(page, {
    "GET /session/current": session(["list-members", "view-member"], ["payment-history"]),
    [`GET /members/${ids.member}`]: fixture("member_detail"),
    "GET /payments/42/history": [fixture("payment_transaction")],
  });

  await page.goto(`/members/${ids.member}`);
  await page.getByRole("button", { name: "Payments", exact: true }).click();
  // The transaction covers April and May 2026: May shows the amount, April points to it.
  // The history is rendered twice (per-year tables and a grid); CSS hides one by screen width.
  await expect(page.getByText("Paid in May 2026").filter({ visible: true })).toBeVisible();
  await expect(page.getByText("150.00 CZK").filter({ visible: true })).toBeVisible();
});
