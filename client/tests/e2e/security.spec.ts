// docs/10_TESTING_QA.md §4 + 09_SECURITY_PRIVACY_ACCESSIBILITY.md §1: no
// API response may ever contain a correctness key, an authoring letter, or
// a listening admin script — those must never leave the server.
import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { answerAllObjectiveModules } from './utils/answerKeys';

// This file is ESM (client/package.json has "type": "module") so __dirname
// isn't defined — derive it from import.meta.url instead.
const __dirname = path.dirname(fileURLToPath(import.meta.url));

test('no API response leaks answer keys or listening scripts across a full journey', async ({ page }) => {
  const leaks: string[] = [];

  // Any admin listening script text appearing verbatim in a response body
  // would be a real leak — collect them all up front to scan for.
  const listening = JSON.parse(readFileSync(path.resolve(__dirname, '../../../seed/listening.json'), 'utf-8')) as {
    stimuli: { text: string }[];
  };
  const adminScripts = listening.stimuli.map((s) => s.text).filter((t) => t.length > 20);

  page.on('response', async (response) => {
    if (!response.url().includes('/api/')) return;
    let body = '';
    try {
      body = await response.text();
    } catch {
      return; // binary (audio) responses — not relevant here
    }
    if (/"key_option_id"/.test(body)) leaks.push(`${response.url()} leaked key_option_id`);
    if (/"authoring_letter"/.test(body)) leaks.push(`${response.url()} leaked authoring_letter`);
    if (/"correct"\s*:\s*(true|false)/.test(body)) leaks.push(`${response.url()} leaked a correct field`);
    for (const script of adminScripts) {
      if (body.includes(script)) leaks.push(`${response.url()} leaked a listening admin script verbatim`);
    }
  });

  // A real (not seeded) journey from the start screen, so the listener
  // captures the actual create-session/module-start/submit-response traffic.
  await page.goto('/');
  await page.getByTestId('start-cta').click();
  await page.getByTestId('worked-example-continue').click();
  await page.getByTestId('screen-objective-test').waitFor({ timeout: 15_000 });
  await answerAllObjectiveModules(page);

  expect(leaks, leaks.join('\n')).toEqual([]);
});
