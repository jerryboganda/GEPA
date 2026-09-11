// docs/10_TESTING_QA.md §4: same journey as journey.full, but every
// interaction goes through the keyboard only — no page.click() anywhere —
// to prove the WCAG 2.2 AA "no keyboard trap / full operability" requirement
// (09_SECURITY_PRIVACY_ACCESSIBILITY.md §4) on the real UI.
import { test, expect } from '@playwright/test';
import { correctOptionIds } from './utils/answerKeys';

async function pressSpace(page: import('@playwright/test').Page) {
  await page.keyboard.press('Space');
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
