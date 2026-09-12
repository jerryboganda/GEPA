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

/**
 * LSN testlets lock their answer options until the stimulus audio has
 * played once (CandidateJourney.tsx's `isListeningLocked`), so option
 * buttons render `disabled`. Waiting through real playback (some stimuli
 * run 20s+) would blow the test timeout many times over across a full
 * journey, so this drives the exact same onEnded -> finishPlayback ->
 * onPlayCompleted path the real UI uses, just by dispatching `ended`
 * directly on the <audio> element instead of waiting in real time.
 */
async function unlockListeningIfNeeded(page: Page): Promise<void> {
  const playBtn = page.getByTestId('audio-play-btn');
  if (!(await playBtn.isVisible().catch(() => false))) return; // not an LSN item

  // Retry the whole play+dispatch cycle rather than trying once and silently
  // giving up: a swallowed failure here used to leave the options genuinely
  // disabled, and clickCorrectOption would still find a data-testid match
  // and try to click it — Playwright's own click() then retries a disabled
  // element for the rest of the test's timeout budget, which is what the
  // multi-minute stalls actually were, not a hang in this function itself.
  for (let attempt = 0; attempt < 5; attempt++) {
    const firstOption = page.locator('[data-testid^="option-"]').first();
    if (!(await firstOption.isDisabled().catch(() => false))) return; // unlocked

    await playBtn.click().catch(() => {});
    await page.evaluate(() => {
      document.querySelectorAll('audio').forEach((el) => el.dispatchEvent(new Event('ended')));
    });
    await page
      .waitForFunction(() => !document.querySelector('[data-testid^="option-"]')?.hasAttribute('disabled'), {
        timeout: 3_000,
      })
      .catch(() => {});
  }
}

/** Clicks the one visible option button whose id is the correct answer. */
export async function clickCorrectOption(page: Page): Promise<void> {
  await unlockListeningIfNeeded(page);
  const correct = correctOptionIds();

  // Reading each button's data-testid one at a time (a separate round-trip
  // per button) left a window for React to swap in a new unit's options
  // mid-scan, occasionally producing a mixed read that matched nothing —
  // a real cause of the flaky "no correct option found" failures. A single
  // batched read is atomic from the page's perspective, and retrying a few
  // times rides out any remaining render-in-progress moment instead of
  // failing on the first transient miss.
  for (let attempt = 0; attempt < 10; attempt++) {
    const optionIds = await page
      .locator('[data-testid^="option-"]')
      .evaluateAll((els) => els.map((el) => el.getAttribute('data-testid')?.replace('option-', '') ?? ''));
    const matchIdx = optionIds.findIndex((id) => id && correct.has(id));
    if (matchIdx >= 0) {
      const optionBtn = page.locator('[data-testid^="option-"]').nth(matchIdx);
      // A click Playwright reports as successful doesn't guarantee React's
      // onClick actually committed the selection — the real cause of past
      // multi-minute hangs was objective-next staying disabled after a
      // "successful" click, because selectedAnswers never actually updated.
      // Verify aria-checked flips and retry the click itself if it didn't,
      // instead of trusting the click alone.
      for (let clickAttempt = 0; clickAttempt < 3; clickAttempt++) {
        await optionBtn.click({ timeout: 8_000 });
        const checked = await optionBtn
          .getAttribute('aria-checked')
          .then((v) => v === 'true')
          .catch(() => false);
        if (checked) return;
        await page.waitForTimeout(200);
      }
      return; // let the caller's own next/submit wait surface a clear failure if this never stuck
    }
    await page.waitForTimeout(200);
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
    // Explicit timeouts (rather than the default, which waits out the rest
    // of the test's budget) turn a still-disabled button into a fast, clear
    // failure naming exactly which click never became actionable.
    const nextBtn = page.getByTestId('objective-next');
    while (await nextBtn.isVisible().catch(() => false)) {
      await nextBtn.click({ timeout: 8_000 });
      await clickCorrectOption(page);
    }

    const submitBtn = page.getByTestId('objective-submit');
    await submitBtn.click({ timeout: 8_000 });
    await page.waitForTimeout(150); // let the next unit (or module_complete) render
  }
  throw new Error('answerAllObjectiveModules exceeded its iteration guard — routing likely stuck');
}
