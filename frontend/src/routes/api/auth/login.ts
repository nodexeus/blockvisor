import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { LOGIN_USER } from 'modules/authentication/const';

export const post = async ({ request }) => {
  const data = await request.formData();
  const { email, password } = Object.fromEntries(data);

  try {
    const response = await axios.post(LOGIN_USER, { email, password });
    const expires = new Date(Date.now() + 1000 * 60 * 60 * 24).toUTCString();

    return {
      headers: {
        'Set-Cookie': `user=${JSON.stringify(
          response.data,
        )}; Expires=${expires}; Path=/; SameSite=Strict; HttpOnly`,
        Location: ROUTES.BROADCASTS,
      },
      status: 302,
    };
  } catch (error) {
    // TODO: Handle API error messages once a real API is set up. Mock API doesn't return errors.
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
