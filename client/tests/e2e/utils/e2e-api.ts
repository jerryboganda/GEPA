// Thin wrapper around the server's test-only fast-forward endpoint
// (`POST /api/test/e2e/sessions`, gated by `E2E_MODE=true` — server/src/api.rs).
// Drives the *real* AssessmentService, so a passing e2e run exercises the
// real routing/scoring engine, not a mock.
import type { Page } from '@playwright/test';

export type StopAfter = 'start' | 'worked_example' | 'ls' | 'rd' | 'lsn' | 'receptive' | 'speaking' | 'writing' | 'full';

export interface SeedResult {
  session_id: string;
  token: string;
  reached: string;
}

export async function seedSession(
  page: Page,
  opts: { stopAfter: StopAfter; answerStrategy?: 'all_correct' | 'all_incorrect'; targetGoal?: string }
): Promise<SeedResult> {
  const res = await page.request.post('/api/test/e2e/sessions', {
    data: {
      stop_after: opts.stopAfter,
      answer_strategy: opts.answerStrategy ?? 'all_correct',
      target_goal: opts.targetGoal,
    },
  });
  if (!res.ok()) {
    throw new Error(`seedSession failed (${res.status()}): ${await res.text()}`);
  }
  return res.json();
}

/** Seed a session, then load the app resumed at that exact point. */
export async function seedAndVisit(page: Page, opts: Parameters<typeof seedSession>[1]): Promise<SeedResult> {
  const seed = await seedSession(page, opts);
  await page.addInitScript(
    ({ sessionId, token }) => {
      localStorage.setItem('gepa_active_session', sessionId);
      localStorage.setItem('gepa_token', token);
    },
    { sessionId: seed.session_id, token: seed.token }
  );
  await page.goto('/');
  return seed;
}
