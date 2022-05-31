import axios from 'axios';
import { LOGIN_USER } from 'modules/authentication/const';

export const post = async ({ request }) => {
  const data = await request.json();
  const { email, password } = data;

  try {
    const response = await axios.post(LOGIN_USER, { email, password });

    return {
      body: response.data,
    };
  } catch (error) {
    return {
      status: error.response.status,
      body: error.response.data,
    };
  }
};
