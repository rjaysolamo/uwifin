'use client';

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { request, type User } from '@/lib/api';

export const DEMO_MODE = process.env.NEXT_PUBLIC_APP_MODE !== 'live';
const demoUser: User = { id: 'demo-rjay', email: 'rjay@example.com', name: 'Rjay Solamo' };

type AuthContextValue = {
  user: User | null; loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, name: string) => Promise<void>;
  logout: () => Promise<void>;
  updateName: (name: string) => void;
};
const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(DEMO_MODE ? demoUser : null);
  const [loading, setLoading] = useState(!DEMO_MODE);
  useEffect(() => {
    // Remove credentials left by the original scaffold. Real sessions use HttpOnly cookies.
    localStorage.removeItem('uwifin_token');
    localStorage.removeItem('uwifin_user');
    if (DEMO_MODE) {
      try {
        const saved = JSON.parse(localStorage.getItem('uwifin-demo-profile') || 'null');
        if (saved && typeof saved.name === 'string') setUser({ ...demoUser, name: saved.name });
      } catch { localStorage.removeItem('uwifin-demo-profile'); }
      return;
    }
    request<User>('/auth/me').then(setUser).catch(() => setUser(null)).finally(() => setLoading(false));
  }, []);

  const login = async (email: string, password: string) => {
    if (DEMO_MODE) throw new Error('Account sign-in is not enabled in this demo. Explore the demo dashboard instead.');
    const result = await request<{ user: User }>('/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) });
    setUser(result.user);
  };
  const register = async (email: string, password: string, name: string) => {
    if (DEMO_MODE) throw new Error('Account registration requires the live API. You can explore the demo without an account.');
    const result = await request<{ user: User }>('/auth/register', { method: 'POST', body: JSON.stringify({ email, password, name }) });
    setUser(result.user);
  };
  const logout = async () => {
    if (!DEMO_MODE) await request('/auth/logout', { method: 'POST' });
    setUser(null);
  };
  const updateName = (name: string) => {
    if (!DEMO_MODE) return;
    setUser((current) => current ? { ...current, name } : current);
    localStorage.setItem('uwifin-demo-profile', JSON.stringify({ name }));
  };
  return <AuthContext.Provider value={{ user, loading, login, register, logout, updateName }}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error('AuthProvider is required.');
  return context;
}
