import cookie from 'cookie';

export const getRefreshToken = (request: Request) =>
  request.headers['X-Refresh-Token'];

export const getTokens = (request: Request) => {
  const credentials = request.headers.get('cookie');
  const data = cookie.parse(credentials);
  const userData = JSON.parse(data.user);

  return {
    accessToken: userData.token,
    refreshToken: userData.refresh,
  };
};
