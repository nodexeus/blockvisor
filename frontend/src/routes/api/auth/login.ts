import axios from 'axios';
import { LOGIN_USER } from 'modules/authentication/const';

export const post = async ({ request }) => {
  const data = await request.json();
  const { email, password } = data;

  try {
    const response = await axios.post(LOGIN_USER, { email, password });
    const expires = new Date(Date.now() + 1000 * 60 * 60 * 24).toUTCString();
    const { refresh, ...rest } = response.data;

    return {
      headers: {
        'Set-Cookie': `refresh=${refresh}; Expires=${expires}; Path=/; SameSite=Strict; HttpOnly`,
      },
      body: rest,
    };
  } catch (error) {
    // TODO: Handle API error messages once a real API is set up. Mock API doesn't return errors.
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
