// WCAG 2.2 AA scan helper (docs/10_TESTING_QA.md §4 `a11y.spec.ts`,
// docs/09_SECURITY_PRIVACY_ACCESSIBILITY.md §4).
import AxeBuilder from '@axe-core/playwright';
import { expect, type Page, type TestInfo } from '@playwright/test';

export async function checkA11y(page: Page, testInfo: TestInfo) {
  const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag22aa']).analyze();
  await testInfo.attach('axe-results', { body: JSON.stringify(results, null, 2), contentType: 'application/json' });
  expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
}
