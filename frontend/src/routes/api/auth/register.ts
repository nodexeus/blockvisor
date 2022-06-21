import { ENDPOINTS } from 'consts/endpoints';

export const post = async ({ request }) => {
  const data = await request.json();
  const { email, password, confirmPassword } = data;

  try {
    const response = await fetch(ENDPOINTS.USERS.CREATE_USER_POST, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email,
        password,
        password_confirm: confirmPassword,
      }),
    });

    if (response.ok) {
      const data = await response.json();

      return {
        status: 200,
        body: data,
      };
    }

    return {
      status: data.status,
      body: data.statusText,
    };
  } catch (error) {
    // TODO: Handle API error messages once a real API is set up. Mock API doesn't return errors.
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
