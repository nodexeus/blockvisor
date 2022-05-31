import axios from 'axios';
import { ENDPOINTS } from 'consts/endpoints';

export const post = async ({ request }) => {
  const data = await request.json();
  const { email, password } = data;

  try {
    const response = await axios.post(ENDPOINTS.AUTHENTICATION.LOGIN_POST, {
      email,
      password,
    });

    return {
      body: response.data,
    };
  } catch (error) {
    // TODO: Handle API error messages once a real API is set up. Mock API doesn't return errors.
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
