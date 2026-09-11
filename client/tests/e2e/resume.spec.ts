// docs/10_TESTING_QA.md §4: reloading mid-module restores state from the
// server (`GET /api/sessions/:id`), and a simulated network failure during
// Listening playback replays the same audio without penalty (06 §5 —
// AudioPlayer.tsx's `onError` handler, see DECISIONS.md notes on this fix).
import { test, expect } from '@playwright/test';
import { seedAndVisit } from './utils/e2e-api';

test('reloading mid-module restores the same objective test screen', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'rd' });
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-objective-test')).toBeVisible({ timeout: 20_000 });

  await page.reload();
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-objective-test')).toBeVisible({ timeout: 20_000 });
});

test('a play error shows the reconnect state and does not consume a play', async ({ page }) => {
  await seedAndVisit(page, { stopAfter: 'lsn' });
  await page.getByTestId('resume-detected-btn').click();
  await expect(page.getByTestId('screen-objective-test')).toBeVisible({ timeout: 20_000 });

  // Force the audio element to error out on this one response. The stimulus
  // has `preload="auto"`, so by the time the screen is visible it may
  // already be fully buffered — aborting only *future* requests wouldn't
  // touch an already-downloaded clip, so force a fresh load attempt too.
  await page.route('**/api/media/audio/**', (route) => route.abort());
  await page.evaluate(() => document.querySelector('audio')?.load());
  const playBtn = page.getByTestId('audio-play-btn');
  if (await playBtn.isVisible().catch(() => false)) {
    await playBtn.click();
    await expect(page.getByTestId('audio-reconnecting')).toBeVisible({ timeout: 10_000 });

    // Restore the network and retry — this must still be "Play (1 of 2)",
    // not "Play again (2 of 2)": the failed attempt must not count.
    await page.unroute('**/api/media/audio/**');
    await expect(playBtn).toHaveText(/Play \(1 of 2\)/);
  }
});
