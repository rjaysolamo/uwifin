import type { CSSProperties, ReactNode } from 'react';
import { createContext, useContext, useEffect, useMemo, useState } from 'react';
import { apiClient, type User } from '@/lib/api';

type AuthContextValue = {
  user: User | null;
  token: string | null;
  loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshUser: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const savedToken = window.localStorage.getItem('uwifin_token');
    const savedUser = window.localStorage.getItem('uwifin_user');

    if (savedToken) {
      setToken(savedToken);
    }

    if (savedUser) {
      try {
        setUser(JSON.parse(savedUser) as User);
      } catch {
        window.localStorage.removeItem('uwifin_user');
      }
    }

    setLoading(false);
  }, []);

  useEffect(() => {
    if (token) {
      window.localStorage.setItem('uwifin_token', token);
    } else {
      window.localStorage.removeItem('uwifin_token');
    }
  }, [token]);

  useEffect(() => {
    if (user) {
      window.localStorage.setItem('uwifin_user', JSON.stringify(user));
    } else {
      window.localStorage.removeItem('uwifin_user');
    }
  }, [user]);

  const refreshUser = async () => {
    if (!token) {
      setUser(null);
      return;
    }

    try {
      const me = await apiClient.getCurrentUser(token);
      setUser(me);
    } catch {
      setToken(null);
      setUser(null);
      window.localStorage.removeItem('uwifin_token');
      window.localStorage.removeItem('uwifin_user');
    }
  };

  const login = async (email: string, password: string) => {
    const session = await apiClient.login(email, password);
    setToken(session);
    const me = await apiClient.getCurrentUser(session);
    setUser(me);
  };

  const register = async (email: string, password: string) => {
    const created = await apiClient.register(email, password);
    if (!created.session) {
      throw new Error('Registration succeeded but no session was returned.');
    }

    setToken(created.session);
    const me = await apiClient.getCurrentUser(created.session);
    setUser(me);
  };

  const logout = () => {
    setToken(null);
    setUser(null);
    window.localStorage.removeItem('uwifin_token');
    window.localStorage.removeItem('uwifin_user');
  };

  const value = useMemo<AuthContextValue>(
    () => ({ user, token, loading, login, register, logout, refreshUser }),
    [user, token, loading],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error('useAuth must be used inside an AuthProvider');
  }
  return ctx;
}
