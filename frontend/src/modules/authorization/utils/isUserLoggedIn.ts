export const isUserLoggedIn = (user) => !!user?.id;

export const isUserVerified = (user) => isUserLoggedIn(user) && user?.verified;
