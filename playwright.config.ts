// SPDX-License-Identifier: RPL-1.5
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  timeout: 120_000,
  workers: 2,
  reporter: [["list"]],
  // GDM_CHROME: reuse an already installed Chromium locally; CI runs `playwright install chromium`
  use: { baseURL: "http://127.0.0.1:1431", reducedMotion: "reduce", launchOptions: { executablePath: process.env.GDM_CHROME || undefined } },
  webServer: { command: "npx vite --mode test --port 1431 --strictPort --host 127.0.0.1", url: "http://127.0.0.1:1431", reuseExistingServer: true },
});
