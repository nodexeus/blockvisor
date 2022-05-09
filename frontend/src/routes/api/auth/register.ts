import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { CREATE_USER } from 'modules/authentication/const';

export const post = async ({ request }) => {
  const data = await request.formData();
  const { email, password, confirmPassword } = Object.fromEntries(data);

  try {
    const user = await axios.post(CREATE_USER, {
      email,
      password,
      password_confirm: confirmPassword,
    });

    // TODO: Check if this flow works on real API. Register doesn't return a token, but login does. Also Mock API doesn't return an error. Check what happens when something fails.
    const expires = new Date(Date.now() + 1000 * 60 * 60 * 24).toUTCString();

    return {
      headers: {
        'Set-Cookie': `user=${JSON.stringify(
          user.data,
        )}; Expires=${expires}; Path=/; SameSite=Strict; HttpOnly`,
        Location: ROUTES.DASHBOARD,
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
