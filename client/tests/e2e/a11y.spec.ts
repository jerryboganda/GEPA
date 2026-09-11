// docs/10_TESTING_QA.md §4 + 09_SECURITY_PRIVACY_ACCESSIBILITY.md §4:
// WCAG 2.2 AA, zero violations, on every screen state. Uses the E2E_MODE
// fast-forward endpoint to land directly on each `ScreenStep` rather than
// re-walking the whole journey once per screen.
import { test } from '@playwright/test';
import { seedAndVisit } from './utils/e2e-api';
import { checkA11y } from './utils/axe';

test('start screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await page.goto('/');
  await checkA11y(page, testInfo);
});

test('worked example screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await page.goto('/');
  await page.getByTestId('start-cta').click();
  await page.getByTestId('screen-worked-example').waitFor({ timeout: 15_000 });
  await checkA11y(page, testInfo);
});

test('objective test screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await seedAndVisit(page, { stopAfter: 'ls' });
  await page.getByTestId('resume-detected-btn').click();
  await page.getByTestId('screen-objective-test').waitFor({ timeout: 20_000 });
  await checkA11y(page, testInfo);
});

test('receptive result screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await seedAndVisit(page, { stopAfter: 'receptive' });
  await page.getByTestId('resume-detected-btn').click();
  await page.getByTestId('screen-receptive-result').waitFor({ timeout: 20_000 });
  await checkA11y(page, testInfo);
});

test('full result screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await seedAndVisit(page, { stopAfter: 'full' });
  await page.getByTestId('resume-detected-btn').click();
  await page.getByTestId('screen-full-result').waitFor({ timeout: 30_000 });
  await checkA11y(page, testInfo);
});

test('admin login screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await page.goto('/admin');
  await checkA11y(page, testInfo);
});

test('reviewer login screen has no WCAG 2.2 AA violations', async ({ page }, testInfo) => {
  await page.goto('/review');
  await checkA11y(page, testInfo);
});
