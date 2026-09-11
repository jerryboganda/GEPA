// Token storage (DECISIONS.md D-021 — server-issued JWTs, no external
// identity provider). Two independent tokens can live in the same browser:
// a candidate token (from `POST /api/sessions`) and a staff token (from
// `POST /api/auth/login`) — they're used on different pages
// (`index.astro` vs `admin.astro`/`review.astro`) so there's no real
// conflict, but `getToken()` prefers staff if both happen to be present.
const CANDIDATE_TOKEN_KEY = 'gepa_token';
const STAFF_TOKEN_KEY = 'gepa_staff_token';
const STAFF_INFO_KEY = 'gepa_staff_info';

export interface StaffInfo {
  email: string;
  role: string;
}

function safeGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeSet(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* private browsing / storage disabled — token just won't persist */
  }
}

function safeRemove(key: string) {
  try {
    localStorage.removeItem(key);
  } catch {}
}

export function saveCandidateToken(token: string) {
  safeSet(CANDIDATE_TOKEN_KEY, token);
}

export function saveStaffSession(token: string, info: StaffInfo) {
  safeSet(STAFF_TOKEN_KEY, token);
  safeSet(STAFF_INFO_KEY, JSON.stringify(info));
}

export function getStaffInfo(): StaffInfo | null {
  const raw = safeGet(STAFF_INFO_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as StaffInfo;
  } catch {
    return null;
  }
}

export function clearStaffSession() {
  safeRemove(STAFF_TOKEN_KEY);
  safeRemove(STAFF_INFO_KEY);
}

export function getToken(): string | null {
  return safeGet(STAFF_TOKEN_KEY) ?? safeGet(CANDIDATE_TOKEN_KEY);
}
