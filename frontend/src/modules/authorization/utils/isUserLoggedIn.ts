export const isUserLoggedIn = (user: UserSession) => !!user?.id;

export const isUserVerified = (user: UserSession) =>
  isUserLoggedIn(user) && user?.verified;
