// Authenticated fetch wrapper — every `/api/*` call the candidate/reviewer/
// admin client makes goes through this so the server's `AuthUser`/`StaffAuth`/
// `AdminAuth` extractors (server/src/auth.rs) have a token to verify.
import { getToken } from './session';

export async function authedFetch(input: string, init: RequestInit = {}): Promise<Response> {
  const token = getToken();
  const headers = new Headers(init.headers ?? {});
  if (token) headers.set('Authorization', `Bearer ${token}`);
  return fetch(input, { ...init, headers });
}

/**
 * Download a file from an admin-only endpoint. A plain `<a href>` can't
 * carry an `Authorization` header, so admin CSV exports (gated behind
 * `AdminAuth`) go through `authedFetch` and a blob URL instead.
 */
export async function authedDownload(path: string, filename: string): Promise<void> {
  const res = await authedFetch(path);
  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
