import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// This file is ESM (client/package.json has "type": "module") so __dirname
// isn't defined — derive it from import.meta.url instead.
const __dirname = path.dirname(fileURLToPath(import.meta.url));

// GEPA v2 e2e suite (docs/10_TESTING_QA.md §4). Chromium only: the fake-media-
// stream flags used for Speaking tests are Chromium-specific, and cross-
// browser coverage isn't asked for. Needs a real Postgres reachable via
// DATABASE_URL — this only runs against the GitHub Actions Postgres service
// container (.github/workflows/ci.yml), never against a local database
// (DECISIONS.md D-021).
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false, // shares one server process; sessions are independent, but keep runs predictable
  // `fullyParallel: false` only serialises tests *within* a file — the
  // chromium-desktop and chromium-mobile projects still ran as separate
  // concurrent workers by default, so their journey.full instances (each
  // dozens of real sequential Postgres round-trips) contended for the same
  // one shared dev server, and whichever request lost that race timed out
  // at a different, seemingly random step each run. One worker matches
  // this suite's actual "one shared server" design.
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: 'http://localhost:4321',
    trace: 'retain-on-failure',
    permissions: ['microphone'],
    launchOptions: {
      args: [
        '--use-fake-device-for-media-stream',
        '--use-fake-ui-for-media-stream',
        '--use-file-for-fake-audio-capture=' + path.resolve(__dirname, 'tests/e2e/fixtures/mic-fixture.wav'),
      ],
    },
  },
  projects: [
    { name: 'chromium-desktop', use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 } } },
    { name: 'chromium-mobile', use: { ...devices['Desktop Chrome'], viewport: { width: 390, height: 844 } } },
  ],
  webServer: [
    {
      command: 'cargo run --bin server',
      cwd: path.resolve(__dirname, '..'),
      url: 'http://localhost:8080/healthz',
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      env: { E2E_MODE: 'true', PORT: '8080', CLIENT_DIST: path.resolve(__dirname, 'dist') },
    },
    {
      command: 'npm run dev',
      cwd: __dirname,
      url: 'http://localhost:4321',
      timeout: 60_000,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
