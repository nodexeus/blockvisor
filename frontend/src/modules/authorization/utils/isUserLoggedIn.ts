import type { UserInfo } from 'utils';

export const isUserLoggedIn = (user: UserSession | UserInfo) => !!user?.id;

export const isUserVerified = (user: UserSession | UserInfo) =>
  isUserLoggedIn(user) && user?.verified;
