// Reads the real seed answer-key file directly (Node fs — this runs in the
// Playwright test process, never shipped to the browser bundle) so
// journey.full/keyboardOnly can drive the *actual* UI and answer every
// objective item correctly, without the server ever telling the client
// which option is right (that would defeat the whole point of the test).
//
// Trick: `option_id`s are effectively unique across the entire item bank
// (sha256-derived per item+letter, docs/08_ITEM_BANK_AND_SEED.md §1), so we
// don't even need to know which item is currently on screen — just collect
// every `key_option_id` from the restricted keys file into one Set, and
// click whichever visible `option-{id}` button is in that set.
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';

// This file is ESM (client/package.json has "type": "module") so __dirname
// isn't defined — derive it from import.meta.url instead.
const __dirname = path.dirname(fileURLToPath(import.meta.url));

let cachedCorrectOptionIds: Set<string> | null = null;

export function correctOptionIds(): Set<string> {
  if (cachedCorrectOptionIds) return cachedCorrectOptionIds;
  const raw = readFileSync(path.resolve(__dirname, '../../../../seed/RESTRICTED_answer_keys.json'), 'utf-8');
  const data = JSON.parse(raw) as { keys: Record<string, { key_option_id: string }> };
  cachedCorrectOptionIds = new Set(Object.values(data.keys).map((k) => k.key_option_id));
  return cachedCorrectOptionIds;
}

/** Clicks the one visible option button whose id is the correct answer. */
export async function clickCorrectOption(page: Page): Promise<void> {
  const correct = correctOptionIds();
  const buttons = page.locator('[data-testid^="option-"]');
  const count = await buttons.count();
  for (let i = 0; i < count; i++) {
    const testId = await buttons.nth(i).getAttribute('data-testid');
    const optionId = testId?.replace('option-', '');
    if (optionId && correct.has(optionId)) {
      await buttons.nth(i).click();
      return;
    }
  }
  throw new Error('No correct option found among visible option buttons — item/answer-key mismatch');
}

/**
 * Drives LS -> RD -> LSN to completion via the real UI, always answering
 * correctly (the component itself auto-advances between modules on
 * `module_complete`, landing on the receptive result screen). Handles both
 * single-item (LS) and testlet (RD/LSN) layouts. Bounded loop — the routing
 * engine's locator hard-cap (04 §1) guarantees this terminates well under
 * 100 iterations across all three modules combined, even in the worst case.
 */
export async function answerAllObjectiveModules(page: Page): Promise<void> {
  for (let guard = 0; guard < 100; guard++) {
    if (await page.getByTestId('screen-objective-test').isHidden().catch(() => true)) return;

    await clickCorrectOption(page);

    // Testlet with more than one item: click Next until the final item, then Submit.
    const nextBtn = page.getByTestId('objective-next');
    while (await nextBtn.isVisible().catch(() => false)) {
      await nextBtn.click();
      await clickCorrectOption(page);
    }

    const submitBtn = page.getByTestId('objective-submit');
    await submitBtn.click();
    await page.waitForTimeout(150); // let the next unit (or module_complete) render
  }
  throw new Error('answerAllObjectiveModules exceeded its iteration guard — routing likely stuck');
}
