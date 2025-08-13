// Token storage utilities
export const TOKEN_KEY = 'adx_auth_token';
export const REFRESH_TOKEN_KEY = 'adx_refresh_token';
export const USER_KEY = 'adx_user';

export const storage = {
  // Token management
  setToken: (token: string) => {
    localStorage.setItem(TOKEN_KEY, token);
  },

  getToken: (): string | null => {
    return localStorage.getItem(TOKEN_KEY);
  },

  setRefreshToken: (refreshToken: string) => {
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  },

  getRefreshToken: (): string | null => {
    return localStorage.getItem(REFRESH_TOKEN_KEY);
  },

  // User data management
  setUser: (user: any) => {
    localStorage.setItem(USER_KEY, JSON.stringify(user));
  },

  getUser: () => {
    const userData = localStorage.getItem(USER_KEY);
    return userData ? JSON.parse(userData) : null;
  },

  // Clear all auth data
  clearAuth: () => {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(USER_KEY);
  },

  // Session storage for temporary data
  setSessionData: (key: string, data: any) => {
    sessionStorage.setItem(key, JSON.stringify(data));
  },

  getSessionData: (key: string) => {
    const data = sessionStorage.getItem(key);
    return data ? JSON.parse(data) : null;
  },

  clearSessionData: (key: string) => {
    sessionStorage.removeItem(key);
  },
};