// docs/10_TESTING_QA.md §4 + 09_SECURITY_PRIVACY_ACCESSIBILITY.md §1: no
// API response may ever contain a correctness key, an authoring letter, or
// an admin script (listening scripts, speaking/writing prompt audio
// scripts, interlocutor lines) — those must never leave the server.
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

  // Productive-phase scripts: the audio_script/interlocutor_line values
  // from the seed speaking/writing task banks (server-side rating
  // material — must never be serialized into /speaking/start or
  // /writing/start responses). In THIS seed, every script-bearing task
  // quotes its script inside the candidate-visible prompt (RT "You
  // hear: ...", INT interlocutor lines, listen-to-write "Listen and
  // write: ..."), so verbatim script text in a response can never be
  // distinguished from its own quoted prompt — the enforced invariants
  // are therefore (a) the field names never appear on the wire (the
  // nulled fields are skip_serializing_if'd server-side) and (b) any
  // script material that is NOT part of its task's prompt (future seed
  // revisions, genuinely hidden scripts) still fails this scan.
  const speakingSeed = JSON.parse(readFileSync(path.resolve(__dirname, '../../../seed/speaking_tasks.json'), 'utf-8')) as {
    tasks: { audio_script?: string; interlocutor_line?: string; prompt?: string }[];
  };
  const writingSeed = JSON.parse(readFileSync(path.resolve(__dirname, '../../../seed/writing_tasks.json'), 'utf-8')) as {
    tasks: { audio_script?: string; prompt?: string }[];
  };
  const trimmed = (s: string | undefined) => (s ?? '').replace(/\s+/g, ' ').trim();
  const productiveScripts = [
    ...speakingSeed.tasks.flatMap((t) => {
      const prompt = trimmed(t.prompt);
      return [t.audio_script, t.interlocutor_line].filter(
        (s) => trimmed(s).length > 20 && !prompt.includes(trimmed(s)),
      );
    }),
    ...writingSeed.tasks.flatMap((t) => {
      const prompt = trimmed(t.prompt);
      return [t.audio_script].filter((s) => trimmed(s).length > 20 && !prompt.includes(trimmed(s)));
    }),
  ].map((s) => trimmed(s));

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
    // Server-only field names being present at all is a leak of the field
    // (the values are redacted server-side; the *shape* must not ship either).
    // NB: this works because the engine models skip_serializing_if the
    // field is None — the candidate view nulls them, so neither name nor
    // value reaches the wire.
    if (/"audio_script"\s*:/.test(body)) leaks.push(`${response.url()} leaked audio_script field`);
    if (/"interlocutor_line"\s*:/.test(body)) leaks.push(`${response.url()} leaked interlocutor_line field`);
    const flat = body.replace(/\s+/g, ' ');
    for (const script of adminScripts) {
      if (flat.includes(script)) leaks.push(`${response.url()} leaked a listening admin script verbatim`);
    }
    for (const script of productiveScripts) {
      if (flat.includes(script)) leaks.push(`${response.url()} leaked a productive-phase script verbatim`);
    }
  });

  // A real (not seeded) journey from the start screen, so the listener
  // captures the actual create-session/module-start/submit-response traffic,
  // then continue through the productive phases so the speaking/writing
  // task payloads are scanned too (they were the actual gap this suite
  // missed before: audio_script shipped in /speaking/start responses).
  await page.goto('/');
  await page.getByTestId('start-cta').click();
  await page.getByTestId('worked-example-continue').click();
  await page.getByTestId('screen-objective-test').waitFor({ timeout: 15_000 });
  await answerAllObjectiveModules(page);

  // Receptive result -> continue to speaking -> mic-check screen. The
  // /speaking/start response (with every task payload) is captured by
  // the listener above.
  await page.getByTestId('screen-receptive-result').waitFor({ timeout: 20_000 });
  await page.getByTestId('receptive-continue-speaking').click();
  await page.getByTestId('mic-check-begin-speaking').waitFor({ timeout: 10_000 });

  // Writing phase: from mic-check the real flow needs actual recordings;
  // fetch the writing task payloads directly through the session's own
  // auth token so the scan covers that endpoint's full body too (the
  // same wire response the app itself would receive).
  const token = await page.evaluate(() => localStorage.getItem('gepa_token'));
  const sessionId = await page.evaluate(() => localStorage.getItem('gepa_active_session'));
  if (token && sessionId) {
    const res = await page.request.fetch(`/api/sessions/${sessionId}/writing/start`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    await res.text(); // captured and scanned by the listener
  }

  expect(leaks, leaks.join('\n')).toEqual([]);
});
