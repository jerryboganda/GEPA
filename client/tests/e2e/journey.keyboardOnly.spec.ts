// docs/10_TESTING_QA.md §4: same journey as journey.full, but every
// interaction goes through the keyboard only — no page.click() anywhere —
// to prove the WCAG 2.2 AA "no keyboard trap / full operability" requirement
// (09_SECURITY_PRIVACY_ACCESSIBILITY.md §4) on the real UI.
import { test, expect } from '@playwright/test';
import { correctOptionIds } from './utils/answerKeys';

async function pressSpace(page: import('@playwright/test').Page) {
  await page.keyboard.press('Space');
}

// LSN testlets lock their radio options until the stimulus audio has played
// once (CandidateJourney.tsx's `isListeningLocked`). Activated via keyboard
// (Tab+Enter) to keep this test's no-mouse guarantee, then the wait for
// real playback is skipped by dispatching `ended` directly — some stimuli
// run 20s+, which would blow the test timeout many times over.
async function unlockListeningIfNeeded(page: import('@playwright/test').Page) {
  const playBtn = page.getByTestId('audio-play-btn');
  if (!(await playBtn.isVisible().catch(() => false))) return;
  const firstOption = page.locator('[data-testid^="option-"]').first();
  if (!(await firstOption.isDisabled().catch(() => false))) return;

  await playBtn.focus();
  await page.keyboard.press('Enter');
  await page.evaluate(() => {
    document.querySelectorAll('audio').forEach((el) => el.dispatchEvent(new Event('ended')));
  });
  await page
    .waitForFunction(() => !document.querySelector('[data-testid^="option-"]')?.hasAttribute('disabled'), {
      timeout: 5_000,
    })
    .catch(() => {});
}

test('candidate can reach the receptive result using only the keyboard', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('screen-start')).toBeVisible();

  // Tab to the start CTA and activate with Enter (consent defaults to checked).
  await page.getByTestId('start-cta').focus();
  await page.keyboard.press('Enter');

  await expect(page.getByTestId('screen-worked-example')).toBeVisible({ timeout: 15_000 });
  await page.getByTestId('worked-example-continue').focus();
  await page.keyboard.press('Enter');

  await expect(page.getByTestId('screen-objective-test')).toBeVisible({ timeout: 15_000 });

  const correct = correctOptionIds();
  for (let guard = 0; guard < 100; guard++) {
    if (!(await page.getByTestId('screen-objective-test').isVisible().catch(() => false))) break;

    // Radiogroup: Tab to the first radio, ArrowDown to cycle, Space to select.
    const buttons = page.locator('[data-testid^="option-"]');
    const count = await buttons.count();
    let selectedIndex = -1;
    for (let i = 0; i < count; i++) {
      const testId = await buttons.nth(i).getAttribute('data-testid');
      if (testId && correct.has(testId.replace('option-', ''))) {
        selectedIndex = i;
        break;
      }
    }
    expect(selectedIndex).toBeGreaterThanOrEqual(0);

    await unlockListeningIfNeeded(page);
    await buttons.first().focus();
    for (let i = 0; i < selectedIndex; i++) {
      await page.keyboard.press('ArrowDown');
    }
    await pressSpace(page);

    const nextBtn = page.getByTestId('objective-next');
    if (await nextBtn.isVisible().catch(() => false)) {
      await nextBtn.focus();
      await page.keyboard.press('Enter');
      continue; // more items in this testlet
    }

    const submitBtn = page.getByTestId('objective-submit');
    await submitBtn.focus();
    await page.keyboard.press('Enter');
    await page.waitForTimeout(150);
  }

  await expect(page.getByTestId('screen-receptive-result')).toBeVisible({ timeout: 20_000 });
});
