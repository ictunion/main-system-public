import { test, expect } from "@playwright/test";
import { fixture, ids, login, mockApi, session } from "./support";

test("admin adds and deletes a dues note on the missing dues page", async ({ page }) => {
  await login(page);
  // The page asks the bank API for the current year by default.
  const year = new Date().getFullYear();
  const note = fixture("note_detail");
  let notes: unknown[] = [];

  const api = await mockApi(page, {
    "GET /session/current": session(["list-members", "manage-notes"], ["payment-history"]),
    "GET /members/current": [fixture("member_summary")],
    [`GET /payments/${year}/missing`]: [fixture("payment_missing_member")],
    [`GET /payments/${year}/commented`]: [],
    "GET /notes/member_dues": () => (notes.length ? [fixture("note_summary")] : []),
    [`GET /notes/member_dues/${ids.member}`]: () => notes,
    [`POST /notes/member_dues/${ids.member}`]: (req) => {
      notes = [{ ...note, body: req.postDataJSON().body }];
      return notes[0];
    },
    [`DELETE /notes/member_dues/${ids.member}/${note.id}`]: () => {
      notes = [];
      return { status: 200, message: "deleted" };
    },
  });

  await page.goto("/missing-dues");
  await expect(page.getByText("Nováková")).toBeVisible();

  await page.getByRole("button", { name: "Add note" }).first().click();
  const drawer = page.getByRole("dialog", { name: "Notes" });
  await drawer.locator("textarea").fill("Will pay next week");
  await drawer.getByRole("button", { name: "Add note" }).click();
  await expect(drawer.getByText("Will pay next week")).toBeVisible();

  page.once("dialog", (d) => d.accept()); // "This permanently deletes the note."
  await drawer.getByText("Delete", { exact: true }).click();
  await expect(drawer.getByText("No notes yet.")).toBeVisible();

  const writes = api.calls.filter((c) => c.method !== "GET");
  expect(writes).toEqual([
    { method: "POST", path: `/notes/member_dues/${ids.member}`, body: { body: "Will pay next week" } },
    { method: "DELETE", path: `/notes/member_dues/${ids.member}/${note.id}`, body: null },
  ]);
});
