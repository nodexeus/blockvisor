import { ENDPOINTS } from 'consts/endpoints';

export const post = async ({ request }) => {
  const data = await request.json();
  const { email, password } = data;

  try {
    const data = await fetch(ENDPOINTS.AUTHENTICATION.LOGIN_POST, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    });

    if (data.ok) {
      const res = await data.json();

      return {
        status: 200,
        body: res,
      };
    }

    return {
      status: data.status,
      body: data.statusText,
    };
  } catch (error) {
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.data ?? JSON.stringify(error.message),
    };
  }
};
