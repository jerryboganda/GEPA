// docs/10_TESTING_QA.md §4: stopping after the receptive result must show
// the "Foundation and Receptive Profile" partial label, and Speaking/Writing
// must never render as measured.
import { test, expect } from '@playwright/test';
import { seedAndVisit } from './utils/e2e-api';

test('finishing at the receptive result shows a labelled partial profile', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'receptive' });

  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-receptive-result')).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText('Foundation and Receptive Profile')).toBeVisible();

  await page.getByTestId('receptive-finish').click();
  await expect(page.getByText('Foundation and Receptive Profile Finalized')).toBeVisible();
  await expect(page.getByText(/not measured/i)).toBeVisible();
});
