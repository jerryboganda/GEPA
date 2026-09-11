// docs/10_TESTING_QA.md §4: full candidate journey, desktop + mobile
// (client.playwright.config.ts runs every spec on both `chromium-desktop`
// and `chromium-mobile` projects automatically).
import { test, expect } from '@playwright/test';
import { answerAllObjectiveModules } from './utils/answerKeys';

test('candidate can complete start -> worked example -> LS/RD/LSN -> receptive -> speaking -> writing -> full result', async ({ page }) => {
  // The longest single journey in the suite (every module, real network
  // round-trips throughout, 7 speaking tasks each with a 5s quality-gate
  // wait) — the default 30s test timeout is tuned for the shorter specs.
  test.setTimeout(150_000);
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
    // The recorder's quality gate needs >=3s of *elapsed 1s ticks*, not wall
    // clock time — 3.5s left no margin against tick-boundary jitter and
    // occasionally landed on only 2 elapsed ticks, tripping quality_failed
    // instead of review. 5s gives a full tick of headroom.
    await page.waitForTimeout(5_000);
    await page.getByTestId('recorder-finish-btn').click();
    await expect(page.getByTestId('recorder-review')).toBeVisible({ timeout: 10_000 });

    // A click Playwright reports as landed doesn't guarantee React's onClick
    // actually fired (the same race fixed in answerKeys.ts for objective
    // options) — verify the screen actually starts moving on and retry the
    // click itself if it doesn't, instead of a fire-and-forget click.
    for (let clickAttempt = 0; clickAttempt < 3; clickAttempt++) {
      await page.getByTestId('speaking-submit').click({ timeout: 8_000 });
      const moved = await page
        .getByTestId('recorder-review')
        .waitFor({ state: 'hidden', timeout: 4_000 })
        .then(() => true)
        .catch(() => false);
      if (moved) break;
    }
  }

  // 7. Writing break -> Writing tasks
  await expect(page.getByTestId('screen-writing-break')).toBeVisible({ timeout: 15_000 });
  await page.getByTestId('writing-break-begin').click();

  for (let i = 0; i < 6; i++) {
    if (!(await page.getByTestId('screen-writing-test').isVisible().catch(() => false))) break;
    await page.getByTestId('writing-textarea').fill(
      'This is a deterministic end to end test response with enough words to satisfy the minimum guidance for this task and allow submission to proceed.'
    );

    // TEMPORARY diagnostic: confirm the textarea value actually landed and
    // the submit button is genuinely enabled before we ever click it.
    const taVal = await page.getByTestId('writing-textarea').inputValue();
    const disabled = await page.getByTestId('writing-submit').isDisabled();
    console.log('[diag] writing textarea len', taVal.length, 'submit disabled', disabled);

    // Same click-succeeded-but-onClick-never-fired race as speaking-submit
    // above — verify the screen actually transitions and retry if not.
    for (let clickAttempt = 0; clickAttempt < 3; clickAttempt++) {
      await page.getByTestId('writing-submit').click({ timeout: 8_000 });
      console.log('[diag] writing-submit clicked, attempt', clickAttempt);
      const moved = await page
        .getByTestId('screen-writing-test')
        .waitFor({ state: 'hidden', timeout: 4_000 })
        .then(() => true)
        .catch(() => false);
      console.log('[diag] moved on?', moved);
      if (moved) break;
    }
  }

  // 8. Full result
  await expect(page.getByTestId('screen-full-result')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId('skill-card-RD')).toBeVisible();
  await expect(page.getByTestId('skill-card-LSN')).toBeVisible();
});
