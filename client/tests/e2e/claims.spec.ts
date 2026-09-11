// docs/10_TESTING_QA.md §4 + 05_SCORING_RESULTS_CLAIMS.md §4: the results
// screen must never contain a forbidden claims phrase, skills must render
// above and larger than the headline, and confidence must be Low/Moderate
// only. This list is kept in sync with 05_SCORING_RESULTS_CLAIMS.md §4 /
// `shared_engine::wording_policy` (no cross-language fixture-sharing
// mechanism exists yet — see the comment in `copy_lint`).
import { test, expect } from '@playwright/test';
import { seedAndVisit } from './utils/e2e-api';

const FORBIDDEN_PHRASES = [
  'cefr-aligned',
  'validated',
  'certified',
  'certificate',
  'your level is',
  'your cefr level',
  'official',
  'guarantee',
  'ielts band',
  'oet grade',
  'toefl score',
  'pte score',
  'predict',
  'high confidence',
  'reliability',
  'accuracy of',
];
// Never flag the one mandatory negative disclaimer that legitimately
// contains "predict" (DECISIONS.md D-012).
const ALLOWED_DISCLAIMER = 'gepa does not predict official exam scores';

test('full result screen contains no forbidden claims wording', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'full' });
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-full-result')).toBeVisible({ timeout: 30_000 });

  const bodyText = ((await page.locator('body').innerText()) || '').toLowerCase();
  const withoutDisclaimer = bodyText.replaceAll(ALLOWED_DISCLAIMER, '');
  for (const phrase of FORBIDDEN_PHRASES) {
    expect(withoutDisclaimer, `found forbidden phrase "${phrase}"`).not.toContain(phrase);
  }
  expect(withoutDisclaimer).not.toMatch(/\b(a1|a2|b1|b2|c1|c2)[+-]/);
  expect(withoutDisclaimer).not.toMatch(/confidence[^.]*\d+%/);
});

test('skill cards render above and larger than the headline block', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'full' });
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-full-result')).toBeVisible({ timeout: 30_000 });

  const skillCard = page.getByTestId('skill-card-RD').first();
  const headline = page.getByTestId('results-headline');
  if (await headline.isVisible().catch(() => false)) {
    const skillBox = await skillCard.boundingBox();
    const headlineBox = await headline.boundingBox();
    expect(skillBox).not.toBeNull();
    expect(headlineBox).not.toBeNull();
    // Skills render above (smaller Y) and taller than the headline block —
    // the visual-prominence rule from 05 §3.1.
    expect(skillBox!.y).toBeLessThan(headlineBox!.y);
    expect(skillBox!.height).toBeGreaterThan(headlineBox!.height);
  }
});

test('confidence badge is Low or Moderate only', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'full' });
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-full-result')).toBeVisible({ timeout: 30_000 });
  await expect(page.getByText(/(Low|Moderate) Confidence/)).toBeVisible();
  await expect(page.getByText('High Confidence')).toHaveCount(0);
});
