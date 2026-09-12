// Reviewer/admin login gate shared by `ReviewerDashboard.tsx` and
// `AdminDashboard.tsx` (DECISIONS.md D-021 — email/password against
// `staff_users`, replacing the earlier Google-sign-in design; no Firebase).
// The server is the real gate (`StaffAuth`/`AdminAuth` extractors,
// server/src/auth.rs) — this hook only controls what the UI shows.
import { useEffect, useState } from 'react';
import { clearStaffSession, getStaffInfo, saveStaffSession, type StaffInfo } from './session';

export function useStaffAuth() {
  const [user, setUser] = useState<StaffInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setUser(getStaffInfo());
    setLoading(false);
  }, []);

  const signIn = async (email: string, password: string) => {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    if (!res.ok) {
      throw new Error('Invalid email or password');
    }
    const data = await res.json();
    const info: StaffInfo = { email: data.email, role: data.role };
    saveStaffSession(data.token, info);
    setUser(info);
  };

  const signOut = () => {
    clearStaffSession();
    setUser(null);
  };

  return { user, loading, signIn, signOut };
}
