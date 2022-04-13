export const post = async () => {
  // TODO: Handle API error messages once a real API is set up. Mock API doesn't return errors.
  return {
    headers: {
      'Set-Cookie': `user=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT`,
    },
  };
};
