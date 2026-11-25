import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: "http://localhost:1234",
    trace: "retain-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    // The bundle imports the compiled .bs.js and config.json, so both must exist.
    command: "npx rescript build && npx webpack-dev-server",
    url: "http://localhost:1234",
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
