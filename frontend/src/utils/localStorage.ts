import { browser } from '$app/env';

export const AUTH_KEY = 'auth';

export const getUserInfo = (): UserInfo | undefined => {
  if (browser) {
    const item = localStorage.getItem(AUTH_KEY);
    if (item) {
      return JSON.parse(item);
    }
  }
  return null;
};

export const saveUserinfo = (value: UserInfo): void => {
  if (browser) {
    localStorage.setItem(AUTH_KEY, JSON.stringify(value));
  }
};

export const deleteUserInfo = () => {
  if (browser) {
    localStorage.removeItem(AUTH_KEY);
  }
};
