// docs/10_TESTING_QA.md §4: full candidate journey, desktop + mobile
// (client.playwright.config.ts runs every spec on both `chromium-desktop`
// and `chromium-mobile` projects automatically).
import { test, expect } from '@playwright/test';
import { answerAllObjectiveModules } from './utils/answerKeys';

test('candidate can complete start -> worked example -> LS/RD/LSN -> receptive -> speaking -> writing -> full result', async ({ page }) => {
  // The longest single journey in the suite (every module, real network
  // round-trips throughout) — the default 30s test timeout is tuned for
  // the shorter specs, not this one.
  test.setTimeout(90_000);
  await page.goto('/');

  // 1. Start screen
  await expect(page.getByTestId('screen-start')).toBeVisible();
  await page.getByTestId('goal-chip-General').click().catch(() => {}); // optional; ignore if id differs
  await page.getByTestId('start-cta').click();

  // 2. Worked example (unscored)
  await expect(page.getByTestId('screen-worked-example')).toBeVisible({ timeout: 15_000 });
  await page.getByTestId('worked-example-continue').click();

  // 3. LS -> RD -> LSN, answering every item correctly via the real UI
  await expect(page.getByTestId('screen-objective-test')).toBeVisible({ timeout: 15_000 });
  await answerAllObjectiveModules(page);

  // 4. Receptive result
  await expect(page.getByTestId('screen-receptive-result')).toBeVisible({ timeout: 20_000 });
  await page.getByTestId('receptive-continue-speaking').click();

  // 5. Mic check -> Speaking
  await expect(page.getByTestId('screen-mic-check')).toBeVisible({ timeout: 15_000 });
  await page.getByTestId('mic-check-begin-speaking').click();

  // 6. Speaking tasks — record and submit each until the module completes.
  // Real prep (15s) + speak (45s) windows would blow the test timeout many
  // times over across several tasks, so this uses the recorder's own real
  // "Skip Prep & Record Now" / "Finish & Review Recording" buttons — the
  // same fast path a real candidate can take — instead of waiting through
  // the full countdowns.
  for (let i = 0; i < 10; i++) {
    if (!(await page.getByTestId('screen-speaking-test').isVisible().catch(() => false))) break;
    await expect(page.getByTestId('recorder-prep').or(page.getByTestId('recorder-recording'))).toBeVisible({ timeout: 10_000 });
    const skipPrepBtn = page.getByTestId('recorder-skip-prep-btn');
    if (await skipPrepBtn.isVisible().catch(() => false)) {
      await skipPrepBtn.click();
    }
    await expect(page.getByTestId('recorder-recording')).toBeVisible({ timeout: 10_000 });
    await page.waitForTimeout(3_500); // clears the recorder's >=3s quality-gate floor
    await page.getByTestId('recorder-finish-btn').click();
    await expect(page.getByTestId('recorder-review')).toBeVisible({ timeout: 10_000 });
    await page.getByTestId('speaking-submit').click();
    await page.waitForTimeout(300);
  }

  // 7. Writing break -> Writing tasks
  await expect(page.getByTestId('screen-writing-break')).toBeVisible({ timeout: 15_000 });
  await page.getByTestId('writing-break-begin').click();

  for (let i = 0; i < 6; i++) {
    if (!(await page.getByTestId('screen-writing-test').isVisible().catch(() => false))) break;
    await page.getByTestId('writing-textarea').fill(
      'This is a deterministic end to end test response with enough words to satisfy the minimum guidance for this task and allow submission to proceed.'
    );
    await page.getByTestId('writing-submit').click();
    await page.waitForTimeout(300);
  }

  // 8. Full result
  await expect(page.getByTestId('screen-full-result')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId('skill-card-RD')).toBeVisible();
  await expect(page.getByTestId('skill-card-LSN')).toBeVisible();
});
