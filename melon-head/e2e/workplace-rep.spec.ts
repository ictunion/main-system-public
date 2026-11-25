import { test, expect } from "@playwright/test";
import { fixture, login, mockApi, session } from "./support";

test("workplace rep sees only their own workplace", async ({ page }) => {
  await login(page);
  await mockApi(page, {
    "GET /session/current": session(["list-own-workplace", "list-own-workplace-members"]),
    "GET /workplaces/mine": fixture("workplace_summary"),
    "GET /workplaces/mine/members": [fixture("workplace_member")],
  });

  await page.goto("/");
  // Nav items are clickable <li>s, not links.
  const nav = page.getByRole("navigation").first();
  await expect(nav.getByText("Workplace Members", { exact: true })).toBeVisible();
  await expect(nav.getByText("Members", { exact: true })).toHaveCount(0);
  await expect(nav.getByText("Workplaces", { exact: true })).toHaveCount(0);

  await nav.getByText("Workplace Members", { exact: true }).click();
  await expect(page.getByRole("heading", { name: "Members of ACME" })).toBeVisible();
  await expect(page.getByText("Nováková")).toBeVisible();

  await page.goto("/members");
  await expect(page.getByText("This page requires at least one of the following roles")).toBeVisible();
});
